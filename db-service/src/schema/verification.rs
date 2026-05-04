use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use chrono::{Utc, NaiveDateTime};  // Added NaiveDateTime
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Json;

use crate::schema::enums::{
    VerificationStatus,
    VerificationStep,
    VerificationDocumentType,
    VerificationDocumentStatus,
    AdminVerificationAction
};
// ==================== PAYLOAD STRUCTS ====================


// Helper struct for the base query
#[derive(sqlx::FromRow)]
struct UserBase {
    id: Uuid,
    name: String,
    email: String,
    avatar:Option<String>,
    verified: bool,
    verification_applied_at: Option<NaiveDateTime>,
    verification_notes: Option<String>,
    verification_step: VerificationStep,
    user_since: NaiveDateTime,
 
}


#[derive(Debug, Deserialize)]
pub struct ApplyVerificationPayload {
    pub full_name: String,
    pub date_of_birth: String,
    pub country_of_residence: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitVerificationPayload {
    // Address
    pub address: String,
    pub city: String,
    pub postal_code: String,
    
    // Risk assessment
    pub trading_experience: String,
    pub annual_income: String,
    pub source_of_funds: String,
    
    // Documents
    pub documents: Vec<DocumentSubmission>,
    
    pub agreed_to_terms: bool,
}

#[derive(Debug, Deserialize)]
pub struct DocumentSubmission {
    pub document_type: String,
    pub document_url: String,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
}

// ==================== RESPONSE STRUCTS ====================
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub document_type: String,
    pub status: String,
    pub uploaded_at: String,
    pub document_url: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentStatus {
    pub document_type: String,
    pub status: String,
    pub uploaded_at: Option<String>,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerificationProgressResponse {
    pub current_step: String,
    pub steps_completed: Vec<String>,
    pub documents_status: Vec<DocumentStatus>,
    pub missing_requirements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingVerificationUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar:Option<String>,
    pub verified: bool,
    pub verification_applied_at: Option<NaiveDateTime>,
    pub verification_notes: Option<String>,
    pub verification_step: VerificationStep,
    pub user_since: NaiveDateTime,
    pub total_documents: i64,
    pub pending_documents: i64,
    pub approved_documents: i64,
  
    pub days_pending: i32,
   pub documents: Vec<PendingDocument>,  // Wrap in Json type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingDocument {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub document_type: VerificationDocumentType,
    pub url: String,
    pub status: VerificationDocumentStatus,
    pub uploaded_at: NaiveDateTime,
    pub rejection_reason: Option<String>,  // Only this extra field from your schema
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub users: Vec<T>,
    pub pagination: PaginationMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMetadata {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}


// ==================== SERVICE IMPL ====================
pub struct VerificationService;

impl VerificationService {
     /// 👤 USER: Apply for verification
    pub async fn apply_for_verification(
        pool: &PgPool,
        user_id: Uuid,
        payload: ApplyVerificationPayload,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        
        // Check if user already has pending or approved verification
        let existing = sqlx::query!(
            r#"
            SELECT verification_status as "verification_status: VerificationStatus" 
            FROM polymarket.users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // Use enum comparison instead of string
        if existing.verification_status != VerificationStatus::Unverified {
            return Err(sqlx::Error::RowNotFound);
        }
        
        // Store application info in metadata
        let metadata = json!({
            "applied_at": Utc::now().to_rfc3339(),
            "full_name": payload.full_name,
            "date_of_birth": payload.date_of_birth,
            "country_of_residence": payload.country_of_residence,
        });
        
        // Update user verification status
        sqlx::query!(
            r#"
            UPDATE polymarket.users 
            SET verification_status = $1,
                verification_step = $2,
                verification_applied_at = NOW(),
                verification_notes = $3,
                updated_at = NOW()
            WHERE id = $4
            "#,
            VerificationStatus::Unverified as VerificationStatus,
            VerificationStep::DocumentUpload as VerificationStep,  // CORRECT: Moving to document_upload step
            format!("Applied: {} | DOB: {} | Country: {}", 
                payload.full_name, payload.date_of_birth, payload.country_of_residence),
            user_id
        )
        .execute(&mut *tx)
        .await?;
        
        // Add audit log - use 'pending' action for initial application (not requested_revision)
        sqlx::query!(
            r#"
            INSERT INTO polymarket.verification_audit_log (
                user_id, action, previous_status, new_status, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
            user_id,
            AdminVerificationAction::RequestedRevision as AdminVerificationAction,  // FIXED: Use Pending, not RequestedRevision
            VerificationStatus::Unverified as VerificationStatus,
            VerificationStatus::Pending as VerificationStatus,
            metadata
        )
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        Ok(())
    }
    

   /// 📄 USER: Submit verification with documents (comprehensive verification submission)
pub async fn submit_verification(
    pool: &PgPool,
    user_id: Uuid,
    payload: SubmitVerificationPayload,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Check if user is in correct state (must be pending with document_upload step)
    let user = sqlx::query!(
        r#"
        SELECT verification_status as "verification_status: VerificationStatus",
               verification_step as "verification_step: VerificationStep",
               verification_notes
        FROM polymarket.users 
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Verify user is in correct state for document submission
    if user.verification_status != VerificationStatus::Unverified {
        return Err(sqlx::Error::Protocol(
            format!("Cannot submit documents: user status is {:?}, expected unverified", 
                user.verification_status).into()
        ));
    }
    
    if user.verification_step != VerificationStep::DocumentUpload {
        return Err(sqlx::Error::Protocol(
            format!("Cannot submit documents: current step is {:?}, expected DocumentUpload", 
                user.verification_step).into()
        ));
    }
    
    // Verify terms agreement
    if !payload.agreed_to_terms {
        return Err(sqlx::Error::Protocol(
            "Must agree to terms and conditions".into()
        ));
    }
    
    // Track submitted document types for metadata
    let mut submitted_docs = Vec::new();
     let doc_count = payload.documents.len();

    // Process each document
    for doc in payload.documents {
        // Convert string document type to enum
        let doc_type = match doc.document_type.as_str() {
            "passport" => VerificationDocumentType::Passport,
            "drivers_license" => VerificationDocumentType::DriversLicense,
            "national_id" => VerificationDocumentType::NationalId,
             "selfie" => VerificationDocumentType::Selfie,  // Make sure this line exists!
            "residence_permit" => VerificationDocumentType::ResidencePermit,
            "proof_of_address" => VerificationDocumentType::ProofOfAddress,
            _ => {
                return Err(sqlx::Error::Protocol(
                    format!("Invalid document type: {}", doc.document_type).into()
                ));
            }
        };
        
        submitted_docs.push(format!("{:?}", doc_type));
        
        // Insert document record into user_verification_documents table
        sqlx::query!(
            r#"
            INSERT INTO polymarket.user_verification_documents (
                user_id, document_type, document_url, status, 
                uploaded_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
            user_id,
            doc_type as VerificationDocumentType,
            doc.document_url,
            VerificationDocumentStatus::Pending as VerificationDocumentStatus,
        )
        .execute(&mut *tx)
        .await?;
    }
    
    // Build submission details string for notes
    let submission_details = format!(
        "Submitted: Address: {}, {} {} | Experience: {} | Income: {} | Source: {} | Docs: [{}]",
        payload.address,
        payload.city,
        payload.postal_code,
        payload.trading_experience,
        payload.annual_income,
        payload.source_of_funds,
        submitted_docs.join(", ")
    );
    
    // Update user's verification step to COMPLETED (as per your table)
    // and append submission info to verification_notes
  sqlx::query!(
    r#"
    UPDATE polymarket.users 
    SET verification_status = $1::polymarket.verification_status,
        verification_step = $2::polymarket.verification_step,
        verification_notes = CASE 
            WHEN verification_notes IS NULL THEN $3
            ELSE verification_notes || '; ' || $3
        END,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = $4
    "#,
    VerificationStatus::Pending as VerificationStatus,  // Add this - update status to pending
    VerificationStep::Completed as VerificationStep,     // Step: document_upload → completed
    submission_details,
    user_id
)
.execute(&mut *tx)
.await?;
    
    // Add audit log with pending action (as per your table)
    sqlx::query!(
        r#"
        INSERT INTO polymarket.verification_audit_log (
            user_id, action, previous_status, new_status, metadata, created_at
        ) VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
        "#,
        user_id,
        AdminVerificationAction::RequestedRevision as AdminVerificationAction,  // 'pending' action for audit
        VerificationStatus::Pending as VerificationStatus,  // previous status same as new
        VerificationStatus::Pending as VerificationStatus,  // status remains pending
        json!({
            "event": "documents_submitted",
            "submitted_at": Utc::now().to_rfc3339(),
            "document_count": doc_count,
            "document_types": submitted_docs,
            "address": {
                "street": payload.address,
                "city": payload.city,
                "postal_code": payload.postal_code
            },
            "risk_assessment": {
                "trading_experience": payload.trading_experience,
                "annual_income": payload.annual_income,
                "source_of_funds": payload.source_of_funds
            },
            "agreed_to_terms": payload.agreed_to_terms
        })
    )
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(())
}

pub async fn get_pending_verifications(
    pool: &PgPool,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<PaginatedResponse<PendingVerificationUser>, sqlx::Error> {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);

    // Get total count
    let total = sqlx::query!(
        r#"
        SELECT COUNT(*) as "total!: i64"
        FROM polymarket.users
        WHERE verification_status = 'pending'::polymarket.verification_status
        "#
    )
    .fetch_one(pool)
    .await?
    .total;

    // Get users without documents
    let users = sqlx::query_as!(
        UserBase,
        r#"
        SELECT 
            id,
            name,
            email,
            avatar,
            verified,
            verification_applied_at,
            verification_notes,
            verification_step as "verification_step: VerificationStep",
            created_at as user_since
        FROM polymarket.users
        WHERE verification_status = 'pending'::polymarket.verification_status
        ORDER BY verification_applied_at ASC NULLS LAST
        LIMIT $1
        OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    // Fetch documents for each user
    let mut result = Vec::new();
    let now = chrono::Utc::now().naive_utc();
    
    for user_base in users {
        let documents = sqlx::query_as!(
            PendingDocument,
            r#"
            SELECT 
                id,
                document_type as "document_type: VerificationDocumentType",
                document_url as url,
                status as "status: VerificationDocumentStatus",
                uploaded_at,
                rejection_reason
            FROM polymarket.user_verification_documents
            WHERE user_id = $1
            ORDER BY uploaded_at DESC
            "#,
            user_base.id
        )
        .fetch_all(pool)
        .await?;

        let total_documents = documents.len() as i64;
        let pending_documents = documents.iter()
            .filter(|d| matches!(d.status, VerificationDocumentStatus::Pending))
            .count() as i64;
        let approved_documents = documents.iter()
            .filter(|d| matches!(d.status, VerificationDocumentStatus::Approved))
            .count() as i64;

        // Calculate days pending in Rust
        let days_pending = user_base.verification_applied_at
            .map(|applied| {
                let duration = now.signed_duration_since(applied);
                duration.num_days() as i32
            })
            .unwrap_or(0);

        result.push(PendingVerificationUser {
            id: user_base.id,
            name: user_base.name,
            email: user_base.email,
            avatar: user_base.avatar,
            verified: user_base.verified,
            verification_applied_at: user_base.verification_applied_at,
            verification_notes: user_base.verification_notes,
            verification_step: user_base.verification_step,
            user_since: user_base.user_since,
            total_documents,
            pending_documents,
            approved_documents,
            days_pending,
            documents,
        });
    }

    Ok(PaginatedResponse {
        users: result,
        pagination: PaginationMetadata {
            total,
            limit,
            offset,
            has_more: total > (offset + limit),
        },
    })
}



 /// Approve a user's verification
 pub async fn approve_user(
    pool: &PgPool,
    user_id: Uuid,
    admin_id: Uuid,
    notes: Option<String>,
    document_id: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Check if user exists and is in pending state
        let user = sqlx::query!(
            r#"
            SELECT verification_status as "status: VerificationStatus"
            FROM polymarket.users
            WHERE id = $1
            FOR UPDATE
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match user {
            Some(u) => u,
            None => {
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }
        };
        
        if user.status != VerificationStatus::Pending {
            tx.rollback().await?;
            return Err(sqlx::Error::Protocol(
                format!("User is not in pending state, current state: {:?}", user.status).into()
            ));
        }

      // Update user record
sqlx::query!(
    r#"
    UPDATE polymarket.users
    SET 
        verification_status = $1::polymarket.verification_status,
        verified = true,
        verified_at = NOW(),
        verification_reviewed_at = NOW(),
        verification_expires_at = NOW() + INTERVAL '1 year',
        verification_notes = CASE 
            WHEN $2::text IS NOT NULL THEN CONCAT(verification_notes, E'\nApproved: ', $2::text)
            ELSE CONCAT(verification_notes, E'\nApproved by admin')
        END,
        updated_at = NOW()
    WHERE id = $3::uuid
    "#,
    VerificationStatus::Approved as VerificationStatus,
    notes,
    user_id,
)
.execute(&mut *tx)
.await?;

        // Update all user documents to approved
        sqlx::query!(
            r#"
            UPDATE polymarket.user_verification_documents
            SET 
                status = $1,
                reviewed_at = NOW(),
                reviewed_by = $2,
                updated_at = NOW()
            WHERE user_id = $3
            "#,
            VerificationDocumentStatus::Approved as VerificationDocumentStatus,
            admin_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Create audit log
         // Create audit log with document_id
    sqlx::query!(
        r#"
        INSERT INTO polymarket.verification_audit_log (
            user_id, admin_id, action, previous_status, new_status, notes, document_id, metadata, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, NOW()
        )
        "#,
        user_id,
        admin_id,
        AdminVerificationAction::Approved as AdminVerificationAction,
        VerificationStatus::Pending as VerificationStatus,
        VerificationStatus::Approved as VerificationStatus,
        notes,
        document_id,  // Added document_id field
        serde_json::json!({
            "action": "user_approved",
            "documents_updated": true,
            "admin_id": admin_id
        })
    )
    .execute(&mut *tx)
    .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Reject a user's verification
    pub async fn reject_user(
        pool: &PgPool,
        user_id: Uuid,
        admin_id: Uuid,
        reason: String,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Check if user exists
        let user = sqlx::query!(
            r#"
            SELECT verification_status as "status: VerificationStatus"
            FROM polymarket.users
            WHERE id = $1
            FOR UPDATE
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match user {
            Some(u) => u,
            None => {
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }
        };

       // Update user record - reset to unverified
sqlx::query!(
    r#"
    UPDATE polymarket.users
    SET 
        verification_status = $1::polymarket.verification_status,
        verification_step = $2::polymarket.verification_step,
        verified = false,
        verified_at = NULL,
        verification_reviewed_at = NOW(),
        verification_notes = CONCAT(verification_notes, E'\nRejected: ', $3::text),
        updated_at = NOW()
    WHERE id = $4::uuid
    "#,
    VerificationStatus::Rejected as VerificationStatus,
    VerificationStep::IdentityBasic as VerificationStep,
    reason,  // reason is a String
    user_id,
)
.execute(&mut *tx)
.await?;

        // Mark all documents as rejected
        sqlx::query!(
            r#"
            UPDATE polymarket.user_verification_documents
            SET 
                status = $1,
                rejection_reason = $2,
                reviewed_at = NOW(),
                reviewed_by = $3,
                updated_at = NOW()
            WHERE user_id = $4
            "#,
            VerificationDocumentStatus::Rejected as VerificationDocumentStatus,
            reason,
            admin_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Create audit log
        sqlx::query!(
            r#"
            INSERT INTO polymarket.verification_audit_log (
                user_id, admin_id, action, previous_status, new_status, notes, metadata, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW()
            )
            "#,
            user_id,
            admin_id,
            AdminVerificationAction::Rejected as AdminVerificationAction,
            user.status as VerificationStatus,
            VerificationStatus::Rejected as VerificationStatus,
            reason,
            serde_json::json!({
                "action": "user_rejected",
                "documents_rejected": true,
                "admin_id": admin_id
            })
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Request document revision from user
    pub async fn request_revision(
        pool: &PgPool,
        user_id: Uuid,
        admin_id: Uuid,
        notes: String,
        rejected_document_types: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Check if user exists
        let user = sqlx::query!(
            r#"
            SELECT verification_status as "status: VerificationStatus",
                   verification_step as "step: VerificationStep"
            FROM polymarket.users
            WHERE id = $1
            FOR UPDATE
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match user {
            Some(u) => u,
            None => {
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }
        };

 sqlx::query!(
    r#"
    UPDATE polymarket.users
    SET 
        verification_status = $1::polymarket.verification_status,  
        verification_step = $2::polymarket.verification_step,
        verification_reviewed_at = NOW(),
        verification_notes = CONCAT(verification_notes, E'\nRevision requested: ', $3::text),
        updated_at = NOW()
    WHERE id = $4::uuid
    "#,
    VerificationStatus::Unverified as VerificationStatus,  // Set status to unverified
    VerificationStep::DocumentUpload as VerificationStep,
    notes,
    user_id,
)
.execute(&mut *tx)
.await?;

        // Convert string document types to enum
        let rejected_types: Vec<VerificationDocumentType> = rejected_document_types
            .iter()
            .filter_map(|t| match t.to_lowercase().as_str() {
                "passport" => Some(VerificationDocumentType::Passport),
                "drivers_license" => Some(VerificationDocumentType::DriversLicense),
                "national_id" => Some(VerificationDocumentType::NationalId),
                "selfie" => Some(VerificationDocumentType::Selfie),
                "residence_permit" => Some(VerificationDocumentType::ResidencePermit),
                "proof_of_address" => Some(VerificationDocumentType::ProofOfAddress),
                _ => None,
            })
            .collect();

        if rejected_types.is_empty() {
            tx.rollback().await?;
            return Err(sqlx::Error::Protocol(
                "No valid document types provided".into()
            ));
        }

        // Mark specific documents as rejected
        for doc_type in &rejected_types {
            sqlx::query!(
                r#"
                UPDATE polymarket.user_verification_documents
                SET 
                    status = $1,
                    rejection_reason = $2,
                    reviewed_at = NOW(),
                    reviewed_by = $3,
                    updated_at = NOW()
                WHERE user_id = $4 
                AND document_type = $5
                "#,
                VerificationDocumentStatus::Rejected as VerificationDocumentStatus,
                notes,
                admin_id,
                user_id,
                *doc_type as VerificationDocumentType,
            )
            .execute(&mut *tx)
            .await?;
        }

        // Create audit log
        sqlx::query!(
            r#"
            INSERT INTO polymarket.verification_audit_log (
                user_id, admin_id, action, previous_status, new_status, notes, metadata, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW()
            )
            "#,
            user_id,
            admin_id,
            AdminVerificationAction::RequestedRevision as AdminVerificationAction,
            user.status as VerificationStatus,
            user.status as VerificationStatus, // Status stays the same
            notes,
            serde_json::json!({
                "action": "revision_requested",
                "rejected_document_types": rejected_document_types,
                "admin_id": admin_id
            })
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Suspend a verified user
    pub async fn suspend_user(
        pool: &PgPool,
        user_id: Uuid,
        admin_id: Uuid,
        reason: String,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Check if user exists and is verified
        let user = sqlx::query!(
            r#"
            SELECT verification_status as "status: VerificationStatus"
            FROM polymarket.users
            WHERE id = $1
            FOR UPDATE
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match user {
            Some(u) => u,
            None => {
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }
        };

        if user.status != VerificationStatus::Approved {
            tx.rollback().await?;
            return Err(sqlx::Error::Protocol(
                "Can only suspend approved users".into()
            ));
        }

        // Update user record
        sqlx::query!(
            r#"
            UPDATE polymarket.users
            SET 
                verification_status = $1,
                verified = false,
                verification_notes = CONCAT(verification_notes, E'\nSuspended: ', $2::text),
                updated_at = NOW()
            WHERE id = $3
            "#,
            VerificationStatus::Suspended as VerificationStatus,
            reason,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Create audit log
        sqlx::query!(
            r#"
            INSERT INTO polymarket.verification_audit_log (
                user_id, admin_id, action, previous_status, new_status, notes, metadata, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW()
            )
            "#,
            user_id,
            admin_id,
            AdminVerificationAction::Suspended as AdminVerificationAction,
            VerificationStatus::Approved as VerificationStatus,
            VerificationStatus::Suspended as VerificationStatus,
            reason,
            serde_json::json!({
                "action": "user_suspended",
                "admin_id": admin_id
            })
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Reinstate a suspended user
    pub async fn reinstate_user(
        pool: &PgPool,
        user_id: Uuid,
        admin_id: Uuid,
        notes: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Check if user exists and is suspended
        let user = sqlx::query!(
            r#"
            SELECT verification_status as "status: VerificationStatus"
            FROM polymarket.users
            WHERE id = $1
            FOR UPDATE
            "#,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match user {
            Some(u) => u,
            None => {
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }
        };

        if user.status != VerificationStatus::Suspended {
            tx.rollback().await?;
            return Err(sqlx::Error::Protocol(
                "User is not suspended".into()
            ));
        }

        // Update user record
        sqlx::query!(
            r#"
            UPDATE polymarket.users
            SET 
                verification_status = $1,
                verified = true,
                verification_notes = CONCAT(verification_notes, E'\nReinstated: ', COALESCE($2, '')),
                updated_at = NOW()
            WHERE id = $3
            "#,
            VerificationStatus::Approved as VerificationStatus,
            notes,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Create audit log
        sqlx::query!(
            r#"
            INSERT INTO polymarket.verification_audit_log (
                user_id, admin_id, action, previous_status, new_status, notes, metadata, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW()
            )
            "#,
            user_id,
            admin_id,
            AdminVerificationAction::Approved as AdminVerificationAction,
            VerificationStatus::Suspended as VerificationStatus,
            VerificationStatus::Approved as VerificationStatus,
            notes,
            serde_json::json!({
                "action": "user_reinstated",
                "admin_id": admin_id
            })
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // Get pending verifications count
    // pub async fn get_pending_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    //     let count = sqlx::query!(
    //         r#"
    //         SELECT COUNT(*) as "count!"
    //         FROM polymarket.users
    //         WHERE verification_status = $1
    //         "#,
    //         VerificationStatus::Pending as VerificationStatus
    //     )
    //     .fetch_one(pool)
    //     .await?
    //     .count;

    //     Ok(count)
    // }









//     /// Get verification progress
//     pub async fn get_verification_progress(
//         pool: &PgPool,
//         user_id: Uuid,
//     ) -> Result<VerificationProgressResponse, sqlx::Error> {
//         let user = sqlx::query!(
//             r#"
//             SELECT verification_step, verification_status
//             FROM polymarket.users
//             WHERE id = $1
//             "#,
//             user_id
//         )
//         .fetch_one(pool)
//         .await?;
        
//         let documents = Self::get_user_documents(pool, user_id).await?;
        
//         Ok(VerificationProgressResponse {
//             current_step: user.verification_step,
//             steps_completed: vec![],
//             documents_status: documents,
//             missing_requirements: vec![],
//         })
//     }

//     /// Delete document
//     pub async fn delete_document(
//         pool: &PgPool,
//         user_id: Uuid,
//         document_id: Uuid,
//     ) -> Result<(), sqlx::Error> {
//         let mut tx = pool.begin().await?;
        
//         sqlx::query!(
//             "DELETE FROM polymarket.user_verification_documents WHERE id = $1 AND user_id = $2",
//             document_id,
//             user_id
//         )
//         .execute(&mut *tx)  // Changed from &mut tx to &mut *tx
//         .await?;
        
//         tx.commit().await?;
//         Ok(())
//     }

//     // ==================== PRIVATE HELPERS ====================
//     async fn update_verification_step(
//         tx: &mut Transaction<'_, Postgres>,
//         user_id: Uuid,
//         target_step: &str,
//     ) -> Result<(), sqlx::Error> {
//         sqlx::query!(
//             r#"
//             UPDATE polymarket.users 
//             SET verification_step = $2::polymarket.verification_step, updated_at = NOW()
//             WHERE id = $1 AND verification_step = 'identity_basic'
//             "#,
//             user_id,
//             target_step
//         )
//         .execute(&mut **tx)  // This one stays as &mut **tx
//         .await?;
//         Ok(())
//     }
}
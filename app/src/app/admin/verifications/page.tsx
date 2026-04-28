"use client";

// pages/admin/verifications.tsx
import React, { useState, useEffect } from 'react';


import { PaginationMetadata, PendingVerificationUser, VerificationService } from '@/utils/interactions/dataGetter';
import UserVerificationCard from '@/components/UserVerificationCard';
import Pagination from '@/components/Pagination';
import ActionModal from '@/components/ActionModal';
import UserDetailsModal from '@/components/UserDetailsModal';
import { Alert, Heading, Text, Stack, Box, Container } from "@chakra-ui/react";
import { FaExclamationTriangle } from "react-icons/fa"
import { toaster } from '@/components/ui/toaster';


export default function VerificationsPage() {
    const [users, setUsers] = useState<PendingVerificationUser[]>([]);
    const [pagination, setPagination] = useState<PaginationMetadata>({
        total: 0,
        limit: 20,
        offset: 0,
        has_more: false,
    });
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedUser, setSelectedUser] = useState<PendingVerificationUser | null>(null);
    const [actionModal, setActionModal] = useState<{
        isOpen: boolean;
        type: 'approve' | 'reject' | 'request_revision' | null;
        userId: string | null;
    }>({ isOpen: false, type: null, userId: null });

    const fetchVerifications = async (offset: number = 0) => {
        setLoading(true);
        setError(null);
        try {
            const data = await VerificationService.getPendingVerifications(20, offset);
            setUsers(data.users);
            setPagination(data.pagination);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to fetch verifications');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchVerifications(0);
    }, []);

    const handlePageChange = (newOffset: number) => {
        fetchVerifications(newOffset);
    };

    const handleAction = async (data: any) => {
        if (!actionModal.userId || !actionModal.type) return;

        let actionPromise;
        let loadingMessage = "";

        switch (actionModal.type) {
            case 'approve':
                actionPromise = VerificationService.approveUser(
                    actionModal.userId,
                    data.notes,
                    data.document_id  // Added document_id parameter
                );
                loadingMessage = "Approving user...";
                break;
            case 'reject':
                actionPromise = VerificationService.rejectUser(actionModal.userId, data.reason);
                loadingMessage = "Rejecting user...";
                break;
            case 'request_revision':
                actionPromise = VerificationService.requestRevision(
                    actionModal.userId,
                    data.notes,
                    data.rejected_document_types || []
                );
                loadingMessage = "Requesting revision...";
                break;
            default:
                return;
        }

        toaster.promise(
            actionPromise,
            {
                loading: { title: loadingMessage },
                success: () => {
                    fetchVerifications(pagination.offset);
                    setActionModal({ isOpen: false, type: null, userId: null });
                    return {
                        title: "Success",
                        description: `User ${actionModal.type === 'approve' ? 'approved' : actionModal.type === 'reject' ? 'rejected' : 'revision requested'} successfully`,
                        closable: true,
                    };
                },
                error: (error) => ({
                    title: "Action Failed",
                    description: error instanceof Error ? error.message : "Please try again",
                    closable: true,
                }),
            }
        );
    };


    if (loading && users.length === 0) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
            </div>
        );
    }

    return (
        <div className="container mx-auto px-50 py-8">

            {/* Header */}
            <Container maxW="7xl" py={8} px={1}>

                {/* Header */}
                <Box mb={8}>
                    <Heading size="lg">Verification Requests</Heading>

                    <Text color="gray.600" mt={2}>
                        {pagination.total} pending{" "}
                        {pagination.total === 1 ? "user" : "users"} waiting for review
                    </Text>
                </Box>

                {/* rest of code unchanged */}

            </Container>

            {/* Error */}
            {error && (
                <Alert.Root status="error" mb={6} borderRadius="md">
                    <FaExclamationTriangle />
                    <Alert.Description>{error}</Alert.Description>
                </Alert.Root>
            )}

            {/* Users */}
            <Stack gap={6}>
                {users.map((user) => (
                    <UserVerificationCard
                        key={user.id}
                        user={user}
                        onViewDetails={() => setSelectedUser(user)}
                        onApprove={() =>
                            setActionModal({
                                isOpen: true,
                                type: "approve",
                                userId: user.id,
                            })
                        }
                        onReject={() =>
                            setActionModal({
                                isOpen: true,
                                type: "reject",
                                userId: user.id,
                            })
                        }
                        onRequestRevision={() =>
                            setActionModal({
                                isOpen: true,
                                type: "request_revision",
                                userId: user.id,
                            })
                        }
                    />
                ))}
            </Stack>

            {/* Pagination */}
            <Box mt={10}>
                <Pagination
                    pagination={pagination}
                    onPageChange={handlePageChange}
                />
            </Box>

            {/* Details Modal */}
            {selectedUser && (
                <UserDetailsModal
                    user={selectedUser}
                    onClose={() => setSelectedUser(null)}
                />
            )}

            {/* Action Modal */}
            <ActionModal
                isOpen={actionModal.isOpen}
                type={actionModal.type}
                user={users.find((u) => u.id === actionModal.userId)}
                onClose={() =>
                    setActionModal({
                        isOpen: false,
                        type: null,
                        userId: null,
                    })
                }
                onSubmit={handleAction}
            />
        </div>
    );
}












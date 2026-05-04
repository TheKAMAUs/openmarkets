"use client";

import { useState, useRef, useEffect } from "react";
import {
    Box,
    Container,
    VStack,
    Heading,
    Text,
    Button,
    Steps,


    Alert,

    Input,
    Field,

    Select,
    Checkbox,


    Icon,
    List,


    Badge,
    Progress,
    HStack,

    createToaster,  // ✅ Import createToaster instead of useToast
} from "@chakra-ui/react";
import { useRouter } from "next/navigation";
import {
    FaCheckCircle,
    FaIdCard,
    FaUserCheck,
    FaShieldAlt,
    FaUpload,
    FaCamera,
    FaHourglassHalf,
    FaTimesCircle,
    FaExclamationTriangle
} from "react-icons/fa";
import { useQuery } from "@tanstack/react-query";
import { UserGetters } from "@/utils/interactions/dataGetter";
import { uploadFileToFirebase } from "../lib/services/firebase";
import { VerificationService } from "@/utils/interactions/dataPosters";
import { createListCollection, } from "@chakra-ui/react";
import { FaCheck } from "react-icons/fa";
import { useMutation } from "@tanstack/react-query";


// Enum types matching PostgreSQL enums
export type VerificationStatus = 'unverified' | 'pending' | 'approved' | 'rejected' | 'expired' | 'suspended';
export type VerificationStep = 'identity_basic' | 'document_upload' | 'liveness_check' | 'address_verification' | 'risk_assessment' | 'completed';
// Update your enum to include selfie
export type VerificationDocumentType =
    | 'passport'
    | 'drivers_license'
    | 'national_id'
    | 'residence_permit'
    | 'proof_of_address'
    | 'selfie';  // Add selfie here

export type VerificationDocumentStatus = 'pending' | 'approved' | 'rejected' | 'expired';

// Map database verification_step to UI step index
const stepToIndex: Record<VerificationStep, number> = {
    'identity_basic': 0,
    'document_upload': 1,
    'liveness_check': 2,
    'address_verification': 3,
    'risk_assessment': 4,
    'completed': 5
};


// Define type
interface SelectItem {
    label: string;
    value: string;
}

// Create collections
const countryCollection = createListCollection<SelectItem>({
    items: [
        { label: "United States", value: "US" },
        { label: "United Kingdom", value: "UK" },
        { label: "Kenya", value: "KE" },
        { label: "Nigeria", value: "NG" },
    ]
});

const idTypeCollection = createListCollection<SelectItem>({
    items: [
        { label: "Passport", value: "passport" },
        { label: "Driver's License", value: "drivers_license" },
        { label: "National ID", value: "national_id" },
        { label: "Residence Permit", value: "residence_permit" },
    ]
});

const tradingExperienceCollection = createListCollection({
    items: [
        { label: "Beginner (Less than 1 year)", value: "beginner" },
        { label: "Intermediate (1-3 years)", value: "intermediate" },
        { label: "Advanced (3+ years)", value: "advanced" },
    ]
});

const annualIncomeCollection = createListCollection({
    items: [
        { label: "$0 - $25,000", value: "0-25000" },
        { label: "$25,000 - $50,000", value: "25000-50000" },
        { label: "$50,000 - $100,000", value: "50000-100000" },
        { label: "$100,000+", value: "100000+" },
    ]
});

const sourceOfFundsCollection = createListCollection({
    items: [
        { label: "Employment", value: "employment" },
        { label: "Business", value: "business" },
        { label: "Investments", value: "investments" },
        { label: "Inheritance", value: "inheritance" },
    ]
});

// Create a toaster instance
const toaster = createToaster({
    placement: "top",
    overlap: true,
    gap: 16,
});


export default function VerificationPage() {
    const router = useRouter();

    const [isSubmitting, setIsSubmitting] = useState(false);
    const [agreedToTerms, setAgreedToTerms] = useState(false);
    const [activeStep, setActiveStep] = useState(0);

    const [tradingExperience, setTradingExperience] = useState('');
    const [annualIncome, setAnnualIncome] = useState('');
    const [sourceOfFunds, setSourceOfFunds] = useState('');
    const Divider = () => <Box width="100%" height="1px" bg={"white"} my={4} />;
    // Use toaster instead of toast
    const showToast = (status: "success" | "error" | "warning" | "info", title: string, description?: string) => {
        toaster.create({
            title,
            description,
            type: status,
            duration: 5000,
        });
    };

    // Example usage:
    const handleSuccess = () => {
        showToast("success", "Success!", "Operation completed successfully");
    };
    // Form state
    const [formData, setFormData] = useState({
        fullName: '',
        dateOfBirth: '',
        countryOfResidence: '',
        idType: '' as VerificationDocumentType,
        address: '',
        city: '',
        postalCode: ''
    });

    // File upload state
    const [documents, setDocuments] = useState<Record<string, File | null>>({
        frontId: null,
        backId: null,
        selfie: null,
        proofOfAddress: null
    });

    const fileInputRefs = {
        frontId: useRef<HTMLInputElement>(null),
        backId: useRef<HTMLInputElement>(null),
        selfie: useRef<HTMLInputElement>(null),
        proofOfAddress: useRef<HTMLInputElement>(null)
    };



    // Fetch user's current verification status
    const { data: userData, refetch, isLoading } = useQuery({
        queryKey: ["userMetadata"],
        queryFn: UserGetters.getUserMetadata,
    });

    // Update active step based on user's verification_step from backend
    useEffect(() => {
        if (userData?.profile_insight?.verification_step) {
            const step = userData.profile_insight.verification_step as VerificationStep;
            setActiveStep(stepToIndex[step] || 0);
        }
    }, [userData]);

    const verificationStatus = userData?.profile_insight?.verification_status as VerificationStatus || 'unverified';
    const currentStep = userData?.profile_insight?.verification_step as VerificationStep || 'identity_basic';

    const steps = [
        { title: "Identity", description: "Basic info" },
        { title: "Documents", description: "Upload ID" },
        { title: "Liveness", description: "Selfie check" },
        { title: "Address", description: "Proof of address" },
        { title: "Risk", description: "Assessment" },
        { title: "Complete", description: "Verified!" },
    ];

    const handleFileChange = (docType: string, file: File | null) => {
        setDocuments(prev => ({ ...prev, [docType]: file }));
    };

    const triggerFileInput = (docType: keyof typeof fileInputRefs) => {
        fileInputRefs[docType].current?.click();
    };
    const { mutateAsync: startVerification, isPending } = useMutation({
        mutationFn: VerificationService.applyForVerification,
    });

    const handleStartVerification = async () => {
        toaster.promise(
            startVerification({
                full_name: formData.fullName,
                date_of_birth: formData.dateOfBirth,
                country_of_residence: formData.countryOfResidence
            }),
            {
                loading: { title: "Starting Verification..." },
                success: () => {
                    setActiveStep(1);
                    refetch();
                    return {
                        title: "Verification Started!",
                        description: "Please complete the KYC process by uploading your documents.",
                        closable: true
                    };
                },
                error: (error) => ({
                    title: "Error",
                    description: error instanceof Error ? error.message : "Failed to start verification",
                    closable: true,
                }),
            }
        );
    };


    // const handleDocumentUpload = async (docType: VerificationDocumentType) => {
    //     let file: File | null = null;
    //     let documentTypeForBackend = docType;

    //     if (docType === 'passport' || docType === 'drivers_license' || docType === 'national_id') {
    //         file = documents.frontId;
    //     } else if (docType === 'selfie') {
    //         file = documents.selfie;
    //         documentTypeForBackend = 'selfie';
    //     } else if (docType === 'residence_permit' || docType === 'proof_of_address') {
    //         file = documents.proofOfAddress;
    //     }

    //     if (!file) {
    //         toast({
    //             title: "No File Selected",
    //             description: "Please select a file to upload",
    //             status: "warning",
    //             duration: 3000,
    //         });
    //         return;
    //     }

    //     setIsSubmitting(true);

    //     try {
    //         const userId = userData?.user_id;
    //         const userEmail = userData?.profile_insight?.email;

    //         if (!userId || !userEmail) {
    //             throw new Error('User ID or Email not found');
    //         }

    //         // Upload to Firebase first
    //         const downloadUrl = await uploadFileToFirebase(
    //             file,
    //             userId,
    //             userEmail,
    //             documentTypeForBackend
    //         );

    //         // Then send to backend using service
    //         const response = await VerificationService.uploadDocument({
    //             documentType: documentTypeForBackend,
    //             documentUrl: downloadUrl,
    //             fileName: file.name,
    //             fileSize: file.size,
    //             mimeType: file.type,
    //         });

    //         toast({
    //             title: "Document Uploaded",
    //             description: `${documentTypeForBackend.replace(/_/g, ' ')} uploaded successfully`,
    //             status: "success",
    //             duration: 3000,
    //         });

    //         // Update local state
    //         setDocuments(prev => {
    //             const updated = { ...prev };
    //             if (docType === 'passport' || docType === 'drivers_license' || docType === 'national_id') {
    //                 updated.frontId = file;
    //             } else if (docType === 'selfie') {
    //                 updated.selfie = file;
    //             } else if (docType === 'residence_permit' || docType === 'proof_of_address') {
    //                 updated.proofOfAddress = file;
    //             }
    //             return updated;
    //         });

    //         // Check if all required docs are uploaded
    //         const allUploaded =
    //             documents.frontId !== null &&
    //             documents.backId !== null &&
    //             documents.selfie !== null &&
    //             documents.proofOfAddress !== null;

    //         if (allUploaded) {
    //             setActiveStep(2);
    //         }

    //     } catch (error: any) {
    //         console.error('Upload failed:', error);
    //         toast({
    //             title: "Upload Failed",
    //             description: error.message || "Please try again",
    //             status: "error",
    //             duration: 3000,
    //         });
    //     } finally {
    //         setIsSubmitting(false);
    //     }
    // };




    const { mutateAsync: submitKYC, isPending: isSubmittingKYC } = useMutation({
        mutationFn: VerificationService.submitVerification,
    });

    const handleSubmitKYC = async () => {
        if (!agreedToTerms) {
            toaster.info({
                title: "Terms Required",
                description: "Please agree to the terms and conditions",
                closable: true,
            });
            return;
        }

        const userId = userData?.user_id;
        const userEmail = userData?.profile_insight?.email;

        if (!userId || !userEmail) {
            toaster.error({
                title: "Error",
                description: "User ID or Email not found",
                closable: true,
            });
            return;
        }

        // Upload ALL documents to Firebase first
        const uploadPromises = [];
        const documentUrls: Record<string, string> = {};
        setIsSubmitting(true);
        // Upload front ID
        if (documents.frontId) {
            uploadPromises.push(
                uploadFileToFirebase(
                    documents.frontId,
                    userId,
                    userEmail,
                    `${formData.idType}_front`
                ).then(url => { documentUrls.frontId = url; })
            );
        }

        // Upload back ID
        if (documents.backId) {
            uploadPromises.push(
                uploadFileToFirebase(
                    documents.backId,
                    userId,
                    userEmail,
                    `${formData.idType}_back`
                ).then(url => { documentUrls.backId = url; })
            );
        }

        // Upload selfie
        if (documents.selfie) {
            uploadPromises.push(
                uploadFileToFirebase(
                    documents.selfie,
                    userId,
                    userEmail,
                    'selfie'
                ).then(url => { documentUrls.selfie = url; })
            );
        }

        // Upload proof of address
        if (documents.proofOfAddress) {
            uploadPromises.push(
                uploadFileToFirebase(
                    documents.proofOfAddress,
                    userId,
                    userEmail,
                    'proof_of_address'
                ).then(url => { documentUrls.proofOfAddress = url; })
            );
        }

        // Wait for all uploads to complete
        await Promise.all(uploadPromises);

        console.log('✅ All documents uploaded to Firebase:', documentUrls);

        // Prepare documents array for backend - filter out any undefined
        const documentsToSubmit = [
            documents.frontId && {
                document_type: `${formData.idType}`,
                document_url: documentUrls.frontId,
                file_name: documents.frontId.name,
                file_size: documents.frontId.size,
                mime_type: documents.frontId.type,
            },
            documents.backId && {
                document_type: `${formData.idType}`,
                document_url: documentUrls.backId,
                file_name: documents.backId.name,
                file_size: documents.backId.size,
                mime_type: documents.backId.type,
            },
            documents.selfie && {
                document_type: 'selfie',
                document_url: documentUrls.selfie,
                file_name: documents.selfie.name,
                file_size: documents.selfie.size,
                mime_type: documents.selfie.type,
            },
            documents.proofOfAddress && {
                document_type: 'proof_of_address',
                document_url: documentUrls.proofOfAddress,
                file_name: documents.proofOfAddress.name,
                file_size: documents.proofOfAddress.size,
                mime_type: documents.proofOfAddress.type,
            }
        ].filter(Boolean) as Array<{
            document_type: string;
            document_url: string;
            file_name: string;
            file_size: number;
            mime_type: string;
        }>;

        console.log('📤 Sending to backend:', documentsToSubmit.map(d => d.document_type));

        toaster.promise(
            submitKYC({
                // Address info
                address: formData.address,
                city: formData.city,
                postal_code: formData.postalCode,

                // Risk assessment
                trading_experience: tradingExperience,
                annual_income: annualIncome,
                source_of_funds: sourceOfFunds,

                // Documents
                documents: documentsToSubmit,

                agreed_to_terms: agreedToTerms
            }),
            {
                loading: { title: "Submitting verification..." },
                success: () => {
                    setActiveStep(4);
                    setIsSubmitting(false);
                    refetch();
                    return {
                        title: "Verification Submitted",
                        description: "Your documents are being reviewed",
                        closable: true,
                    };
                },
                error: (error) => ({
                    title: "Submission Failed",
                    description: error instanceof Error ? error.message : "Please try again",
                    closable: true,
                }),
            }
        );
    };




    // Render based on verification_status
    const renderStatusBanner = () => {
        switch (verificationStatus) {
            case 'approved':
                return (
                    <Alert.Root status="success" borderRadius="md" mb={6}>
                        <Alert.Indicator />
                        <Box flex="1">
                            <Alert.Title>Verified!</Alert.Title>
                            <Alert.Description>
                                Your account is fully verified. You have access to all features.
                            </Alert.Description>
                        </Box>
                        <Button onClick={() => router.push('/profile')} size="sm">
                            Go to Profile
                        </Button>
                    </Alert.Root>
                );

            case 'rejected':
                return (
                    <Alert.Root status="error" borderRadius="md" mb={6}>
                        <Alert.Indicator />
                        <Box flex="1">
                            <Alert.Title>Verification Rejected</Alert.Title>
                            <Alert.Description>
                                {userData?.profile_insight?.verification_notes || 'Please contact support for details'}
                            </Alert.Description>
                        </Box>
                        <Button onClick={() => setActiveStep(1)} size="sm" colorScheme="red">
                            Try Again
                        </Button>
                    </Alert.Root>
                );

            case 'pending':
                return (
                    <Alert.Root status="warning" borderRadius="md" mb={6}>
                        <Alert.Indicator />
                        <Box flex="1">
                            <Alert.Title>Under Review</Alert.Title>
                            <Alert.Description>
                                Your documents are being reviewed. This usually takes 1-2 business days.
                            </Alert.Description>
                        </Box>
                        <Badge colorScheme="yellow" p={2}>Pending</Badge>
                    </Alert.Root>
                );

            case 'expired':
                return (
                    <Alert.Root status="info" borderRadius="md" mb={6}>
                        <Alert.Indicator />
                        <Box flex="1">
                            <Alert.Title>Verification Expired</Alert.Title>
                            <Alert.Description>
                                Please renew your verification documents.
                            </Alert.Description>
                        </Box>
                        <Button onClick={() => setActiveStep(1)} size="sm">
                            Renew Now
                        </Button>
                    </Alert.Root>
                );

            case 'suspended':
                return (
                    <Alert.Root status="error" borderRadius="md" mb={6}>
                        <Alert.Indicator as={FaExclamationTriangle} />
                        <Box flex="1">
                            <Alert.Title>Account Suspended</Alert.Title>
                            <Alert.Description>
                                Please contact support for assistance.
                            </Alert.Description>
                        </Box>
                    </Alert.Root>
                );

            default:
                return null;
        }
    };




    const renderStepContent = () => {
        // Show loading state
        if (isLoading) {
            return (
                <VStack gap={6} py={10}>
                    <Progress.Root size="xs" value={null} width="100%">
                        <Progress.Track>
                            <Progress.Range />
                        </Progress.Track>
                    </Progress.Root>
                    <Text>Loading verification status...</Text>
                </VStack>
            );
        }

        // If already approved, show success
        if (verificationStatus === 'approved') {
            return (
                <VStack gap={6} align="stretch" textAlign="center" py={8}>
                    <Icon as={FaCheckCircle} boxSize={16} color="green.500" />
                    <Heading size="md">Verification Complete!</Heading>
                    <Text>
                        Your identity has been successfully verified.
                        You now have full access to all platform features.
                    </Text>

                    <List.Root gap={2} textAlign="left" bg="green.50" p={4} borderRadius="md">
                        <List.Item>
                            <List.Indicator as={FaCheckCircle} color="green.500" />
                            Make deposits and withdrawals
                        </List.Item>
                        <List.Item>
                            <List.Indicator as={FaCheckCircle} color="green.500" />
                            Start trading
                        </List.Item>
                        <List.Item>
                            <List.Indicator as={FaCheckCircle} color="green.500" />
                            Access advanced features
                        </List.Item>
                    </List.Root>

                    <Button
                        colorScheme="blue"
                        size="lg"
                        onClick={() => router.push("/profile")}
                    >
                        Go to Profile
                    </Button>
                </VStack>
            );
        }

        // If pending, show waiting screen
        if (verificationStatus === 'pending') {
            return (
                <VStack gap={6} align="stretch" textAlign="center" py={8}>
                    <Icon as={FaHourglassHalf} boxSize={16} color="orange.500" />
                    <Heading size="md">Under Review</Heading>
                    <Text>
                        Your documents are being reviewed by our team.
                        This usually takes 1-2 business days.
                    </Text>

                    <Progress.Root size="xs" value={null} colorScheme="orange" width="100%">
                        <Progress.Track>
                            <Progress.Range />
                        </Progress.Track>
                    </Progress.Root>

                    <Alert.Root status="warning" borderRadius="md">
                        <Alert.Indicator />
                        <Alert.Content>
                            <Alert.Title>Notification</Alert.Title>
                            <Alert.Description>
                                You'll receive an email notification once the verification is complete.
                            </Alert.Description>
                        </Alert.Content>
                    </Alert.Root>

                    <Button onClick={() => router.push("/profile")} variant="outline">
                        Back to Profile
                    </Button>
                </VStack>
            );
        }

        // Regular flow for unverified users
        switch (activeStep) {
            case 0: // identity_basic
                return (
                    <VStack gap={6} align="stretch">
                        <Box textAlign="center" py={6}>
                            <Icon as={FaIdCard} boxSize={16} color="blue.500" mb={4} />
                            <Heading size="md" mb={2}>
                                Identity Verification Required
                            </Heading>
                            <Text color="gray.600">
                                To comply with regulations and ensure platform security,
                                we need to verify your identity before you can:
                            </Text>
                        </Box>

                        <List.Root gap={3}>
                            <List.Item>
                                <List.Indicator as={FaCheckCircle} color="green.500" />
                                Make deposits and withdrawals
                            </List.Item>
                            <List.Item>
                                <List.Indicator as={FaCheckCircle} color="green.500" />
                                Trade on the platform
                            </List.Item>
                            <List.Item>
                                <List.Indicator as={FaCheckCircle} color="green.500" />
                                Access full account features
                            </List.Item>
                        </List.Root>

                        <Divider />

                        <Alert.Root status="info" borderRadius="md">
                            <Alert.Indicator />
                            <Alert.Content>
                                <Alert.Title>Why we verify</Alert.Title>
                                <Alert.Description>
                                    Identity verification helps us prevent fraud and maintain
                                    a secure trading environment for all users.
                                </Alert.Description>
                            </Alert.Content>
                        </Alert.Root>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>Full Name</Field.Label>
                            <Input
                                placeholder="Enter your full legal name"
                                value={formData.fullName}
                                onChange={(e) => setFormData({ ...formData, fullName: e.target.value })}
                            />
                            <Field.HelperText>As it appears on your ID</Field.HelperText>
                        </Field.Root>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>Date of Birth</Field.Label>
                            <Input
                                type="date"
                                value={formData.dateOfBirth}
                                onChange={(e) => setFormData({ ...formData, dateOfBirth: e.target.value })}
                            />
                        </Field.Root>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>Country of Residence</Field.Label>
                            <Select.Root
                                value={formData.countryOfResidence ? [formData.countryOfResidence] : []}
                                onValueChange={(e) => setFormData({
                                    ...formData,
                                    countryOfResidence: e.value[0] || ''
                                })}
                                collection={countryCollection}
                            >
                                <Select.Trigger>
                                    <Select.ValueText placeholder="Select country" />
                                </Select.Trigger>
                                <Select.Content>
                                    {countryCollection.items.map((item) => (
                                        <Select.Item item={item} key={item.value}>
                                            {item.label}
                                        </Select.Item>
                                    ))}
                                </Select.Content>
                            </Select.Root>
                        </Field.Root>

                        <Button
                            colorScheme="blue"
                            size="lg"
                            onClick={handleStartVerification}
                            loading={isSubmitting}
                            loadingText="Starting..."
                            disabled={!formData.fullName || !formData.dateOfBirth || !formData.countryOfResidence}
                        >
                            Continue
                        </Button>
                    </VStack>
                );
            case 1: // document_upload
                return (
                    <VStack gap={6} align="stretch">
                        <Heading size="md">Upload Identification Documents</Heading>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>ID Type</Field.Label>
                            <Select.Root
                                value={formData.idType ? [formData.idType] : []}
                                onValueChange={(e) => setFormData({
                                    ...formData,
                                    idType: e.value[0] as VerificationDocumentType
                                })}
                                collection={idTypeCollection}
                            >
                                <Select.Trigger>
                                    <Select.ValueText placeholder="Select ID type" />
                                </Select.Trigger>
                                <Select.Content>
                                    {idTypeCollection.items.map((item: SelectItem) => (
                                        <Select.Item item={item} key={item.value}>
                                            {item.label}
                                        </Select.Item>
                                    ))}
                                </Select.Content>
                            </Select.Root>
                        </Field.Root>

                        {/* Front of ID */}
                        <Box
                            borderWidth="2px"
                            borderStyle="dashed"
                            borderColor={documents.frontId ? 'green.500' : "gray.800"}
                            borderRadius="lg"
                            p={6}
                            textAlign="center"
                            cursor="pointer"
                            onClick={() => triggerFileInput('frontId')}
                        >
                            <input
                                type="file"
                                ref={fileInputRefs.frontId}
                                style={{ display: 'none' }}
                                accept="image/*,.pdf"
                                onChange={(e) => {
                                    const file = e.target.files?.[0] || null;
                                    handleFileChange('frontId', file);
                                }}
                            />
                            <Icon as={FaUpload} boxSize={8} color={documents.frontId ? 'green.500' : 'gray.400'} />
                            <Text mt={2} fontWeight={documents.frontId ? 'bold' : 'normal'}>
                                {documents.frontId ? documents.frontId.name : 'Upload Front of ID'}
                            </Text>
                            {documents.frontId && (
                                <Button
                                    size="xs"
                                    colorScheme="red"
                                    mt={2}
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        handleFileChange('frontId', null);
                                    }}
                                >
                                    Remove
                                </Button>
                            )}
                        </Box>

                        {/* Back of ID */}
                        <Box
                            borderWidth="2px"
                            borderStyle="dashed"
                            borderColor={documents.backId ? 'green.500' : "gray.800"}
                            borderRadius="lg"
                            p={6}
                            textAlign="center"
                            cursor="pointer"
                            onClick={() => triggerFileInput('backId')}
                        >
                            <input
                                type="file"
                                ref={fileInputRefs.backId}
                                style={{ display: 'none' }}
                                accept="image/*,.pdf"
                                onChange={(e) => {
                                    const file = e.target.files?.[0] || null;
                                    handleFileChange('backId', file);
                                }}
                            />
                            <Icon as={FaUpload} boxSize={8} color={documents.backId ? 'green.500' : 'gray.400'} />
                            <Text mt={2}>{documents.backId ? documents.backId.name : 'Upload Back of ID'}</Text>
                            {documents.backId && (
                                <Button
                                    size="xs"
                                    colorScheme="red"
                                    mt={2}
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        handleFileChange('backId', null);
                                    }}
                                >
                                    Remove
                                </Button>
                            )}
                        </Box>

                        {/* Selfie */}
                        <Box
                            borderWidth="2px"
                            borderStyle="dashed"
                            borderColor={documents.selfie ? 'green.500' : "gray.800"}
                            borderRadius="lg"
                            p={6}
                            textAlign="center"
                            cursor="pointer"
                            onClick={() => triggerFileInput('selfie')}
                        >
                            <input
                                type="file"
                                ref={fileInputRefs.selfie}
                                style={{ display: 'none' }}
                                accept="image/*"
                                capture="user"
                                onChange={(e) => {
                                    const file = e.target.files?.[0] || null;
                                    handleFileChange('selfie', file);
                                }}
                            />
                            <Icon as={FaCamera} boxSize={8} color={documents.selfie ? 'green.500' : 'gray.400'} />
                            <Text mt={2}>{documents.selfie ? documents.selfie.name : 'Take Selfie with ID'}</Text>
                            {documents.selfie && (
                                <Button
                                    size="xs"
                                    colorScheme="red"
                                    mt={2}
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        handleFileChange('selfie', null);
                                    }}
                                >
                                    Remove
                                </Button>
                            )}
                        </Box>

                        <Button
                            colorScheme="blue"
                            size="lg"
                            onClick={() => setActiveStep(3)}
                            disabled={!documents.frontId || !documents.backId || !documents.selfie || !formData.idType}
                        >
                            Continue to Liveness
                        </Button>
                    </VStack>
                );

            // case 2: // liveness_check
            //     return (
            //         <VStack gap={6} align="stretch">
            //             <Heading size="md">Liveness Check</Heading>
            //             <Text>Please take a short video selfie to verify you're a real person.</Text>

            //             <Box
            //                 borderWidth="2px"
            //                 borderStyle="dashed"
            //                 borderColor={borderColor}
            //                 borderRadius="lg"
            //                 p={10}
            //                 textAlign="center"
            //             >
            //                 <Icon as={FaCamera} boxSize={12} color="blue.500" />
            //                 <Text mt={4}>Click to start recording</Text>
            //                 <Button mt={4} colorScheme="blue" leftIcon={<FaCamera />}>
            //                     Start Recording
            //                 </Button>
            //             </Box>

            //             <HStack gap={4}>
            //                 <Button variant="outline" onClick={() => setActiveStep(1)}>
            //                     Back
            //                 </Button>
            //                 <Button
            //                     colorScheme="blue"
            //                     flex={1}
            //                     onClick={() => setActiveStep(3)}
            //                 >
            //                     Continue to Address
            //                 </Button>
            //             </HStack>
            //         </VStack>
            //     );

            case 3: // address_verification
                return (
                    <VStack gap={6} align="stretch">
                        <Heading size="md">Address Verification</Heading>

                        {/* ✅ Fixed: Address fields with Field.Root */}
                        <Field.Root required>
                            <Field.Label>Street Address</Field.Label>
                            <Input
                                placeholder="Enter your street address"
                                value={formData.address}
                                onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                            />
                        </Field.Root>

                        <Field.Root required>
                            <Field.Label>City</Field.Label>
                            <Input
                                placeholder="City"
                                value={formData.city}
                                onChange={(e) => setFormData({ ...formData, city: e.target.value })}
                            />
                        </Field.Root>

                        <Field.Root required>
                            <Field.Label>Postal Code</Field.Label>
                            <Input
                                placeholder="Postal code"
                                value={formData.postalCode}
                                onChange={(e) => setFormData({ ...formData, postalCode: e.target.value })}
                            />
                        </Field.Root>

                        <Box
                            borderWidth="2px"
                            borderStyle="dashed"
                            borderColor={documents.proofOfAddress ? 'green.500' : "gray.800"}
                            borderRadius="lg"
                            p={6}
                            textAlign="center"
                            cursor="pointer"
                            onClick={() => triggerFileInput('proofOfAddress')}
                        >
                            <input
                                type="file"
                                ref={fileInputRefs.proofOfAddress}
                                style={{ display: 'none' }}
                                accept="image/*,.pdf"
                                onChange={(e) => handleFileChange('proofOfAddress', e.target.files?.[0] || null)}
                            />
                            <Icon as={FaUpload} boxSize={8} color={documents.proofOfAddress ? 'green.500' : 'gray.400'} />
                            <Text mt={2}>
                                {documents.proofOfAddress ? documents.proofOfAddress.name : 'Upload Proof of Address'}
                            </Text>
                            <Text fontSize="sm" color="gray.500" mt={1}>
                                Utility bill, bank statement, or government letter (last 3 months)
                            </Text>
                        </Box>

                        <HStack gap={4}>
                            <Button variant="outline" onClick={() => setActiveStep(2)}>
                                Back
                            </Button>
                            <Button
                                colorScheme="blue"
                                flex={1}
                                onClick={() => setActiveStep(4)}
                                disabled={!formData.address || !formData.city || !formData.postalCode || !documents.proofOfAddress}
                            >
                                Continue to Risk Assessment
                            </Button>
                        </HStack>
                    </VStack>
                );

            case 4: // risk_assessment
                return (
                    <VStack gap={6} align="stretch">
                        <Heading size="md">Risk Assessment</Heading>

                        <Alert.Root status="info">
                            <Alert.Indicator />
                            <Alert.Content>
                                <Alert.Title>Information</Alert.Title>
                                <Alert.Description>
                                    Please answer a few questions to help us assess your trading experience.
                                </Alert.Description>
                            </Alert.Content>
                        </Alert.Root>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>Trading Experience</Field.Label>
                            <Select.Root
                                value={tradingExperience ? [tradingExperience] : []}
                                onValueChange={(e) => setTradingExperience(e.value[0])}
                                collection={tradingExperienceCollection}
                            >
                                <Select.Trigger>
                                    <Select.ValueText placeholder="Select experience level" />
                                </Select.Trigger>
                                <Select.Content>
                                    {tradingExperienceCollection.items.map((item: { label: string; value: string }) => (
                                        <Select.Item item={item} key={item.value}>
                                            {item.label}
                                        </Select.Item>
                                    ))}
                                </Select.Content>
                            </Select.Root>
                        </Field.Root>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>Annual Income</Field.Label>
                            <Select.Root
                                value={annualIncome ? [annualIncome] : []}
                                onValueChange={(e) => setAnnualIncome(e.value[0])}
                                collection={annualIncomeCollection}
                            >
                                <Select.Trigger>
                                    <Select.ValueText placeholder="Select range" />
                                </Select.Trigger>
                                <Select.Content>
                                    {annualIncomeCollection.items.map((item: { label: string; value: string }) => (
                                        <Select.Item item={item} key={item.value}>
                                            {item.label}
                                        </Select.Item>
                                    ))}
                                </Select.Content>
                            </Select.Root>
                        </Field.Root>

                        {/* ✅ Fixed: Field.Root replaces FormControl */}
                        <Field.Root required>
                            <Field.Label>Source of Funds</Field.Label>
                            <Select.Root
                                value={sourceOfFunds ? [sourceOfFunds] : []}
                                onValueChange={(e) => setSourceOfFunds(e.value[0])}
                                collection={sourceOfFundsCollection}
                            >
                                <Select.Trigger>
                                    <Select.ValueText placeholder="Select primary source" />
                                </Select.Trigger>
                                <Select.Content>
                                    {sourceOfFundsCollection.items.map((item: { label: string; value: string }) => (
                                        <Select.Item item={item} key={item.value}>
                                            {item.label}
                                        </Select.Item>
                                    ))}
                                </Select.Content>
                            </Select.Root>
                        </Field.Root>


                        <Checkbox.Root
                            checked={agreedToTerms}
                            onCheckedChange={(e) => setAgreedToTerms(e.checked === true)}
                        >
                            <Checkbox.HiddenInput />
                            <Checkbox.Control />
                            <Checkbox.Label>
                                I confirm that all information provided is accurate and complete
                            </Checkbox.Label>
                        </Checkbox.Root>

                        <Checkbox.Root>
                            <Checkbox.HiddenInput />
                            <Checkbox.Control />
                            <Checkbox.Label>
                                I agree to the <Button variant="ghost" colorScheme="blue">Terms of Service</Button>
                                {' '}and <Button variant="ghost" colorScheme="blue">Privacy Policy</Button>
                            </Checkbox.Label>
                        </Checkbox.Root>

                        <HStack gap={4}>
                            <Button variant="outline" onClick={() => setActiveStep(3)}>
                                Back
                            </Button>
                            <Button
                                colorScheme="blue"
                                size="lg"
                                flex={1}
                                onClick={handleSubmitKYC}
                                loading={isSubmitting}
                                loadingText="Submitting..."
                                disabled={!agreedToTerms || !tradingExperience || !annualIncome || !sourceOfFunds}
                            >
                                Submit for Review
                            </Button>
                        </HStack>
                    </VStack>
                );
            default:
                return null;
        }
    };

    return (
        <Container maxW="container.md" py={10}>
            <VStack gap={8} align="stretch">
                <Box textAlign="center">
                    <Icon as={FaShieldAlt} boxSize={12} color="blue.500" mb={4} />
                    <Heading size="xl" mb={2}>
                        Account Verification
                    </Heading>
                    <Text color="gray.600">
                        Secure your account and unlock full platform access
                    </Text>
                </Box>

                {/* Status Banner */}
                {renderStatusBanner()}

                {/* Progress Steps - Only show for unverified/pending */}
                {verificationStatus !== 'approved' && verificationStatus !== 'rejected' && (
                    <Steps.Root zIndex={activeStep} mb={6}>  {/* Changed from index to zIndex */}
                        {steps.map((step, index) => (
                            <Steps.Item key={index} index={index}>  {/* Keep index here for Items */}
                                <Steps.Indicator>
                                    {index < activeStep ? (
                                        <Icon as={FaCheck} color="green.500" />  // Checkmark for completed
                                    ) : (
                                        <Steps.Number />  // Number for pending/active
                                    )}
                                </Steps.Indicator>

                                <Steps.Content index={index}>
                                    <Steps.Title>{step.title}</Steps.Title>
                                    <Steps.Description>{step.description}</Steps.Description>
                                </Steps.Content>

                                {/* Don't add separator after last item */}
                                {index < steps.length - 1 && <Steps.Separator />}
                            </Steps.Item>
                        ))}
                    </Steps.Root>
                )}

                {/* Main Content */}
                <Box
                    bg={"gray.300"}
                    p={8}
                    borderRadius="lg"
                    borderWidth="1px"
                    borderColor={"gray.800"}
                    shadow="sm"
                >
                    {renderStepContent()}
                </Box>

                {/* Help Section */}
                <Box textAlign="center" fontSize="sm" color="gray.500">
                    <Text>
                        Need help? Contact{" "}
                        <Button variant="ghost" colorScheme="blue">
                            support@polymarket.com
                        </Button>
                    </Text>
                </Box>
            </VStack>
        </Container>
    );
}
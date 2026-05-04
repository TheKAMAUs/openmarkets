

// components/ChildMarketModal.tsx
import React, { useState, useEffect, useRef } from 'react';
import { X, Upload } from 'lucide-react';
import {
    Box,
    Flex,
    Heading,
    Text,
    VStack,
    HStack,
    Button,
    Input,
    Textarea,
    Field,
    NativeSelect,
    Badge,
    SimpleGrid,
    IconButton,
    Spinner,
} from "@chakra-ui/react";

import useUserInfo from '@/hooks/useUserInfo';
import { uploadFileToFirebase } from '@/app/lib/services/firebase';

export interface ChildMarketData {
    name?: string;
    question?: string;
    logo?: string[];
    liquidity?: number;
    market_expiry?: string;
    slug?: string;
    category?: string;
    resolution_criteria?: string;
}

interface Props {
    isOpen: boolean;
    isEditing: boolean;
    initialData?: ChildMarketData;
    onClose: () => void;
    onSave: (data: ChildMarketData) => void;
    parentMarketName?: string;
}

const CATEGORIES = [
    "Politics",
    "Sports",
    "Crypto",
    "Economics",
    "Technology",
    "Entertainment",
    "Science",
    "World News",
    "Business",
    "Other",
];

export default function ChildMarketModal({
    isOpen,
    isEditing,
    initialData,
    onClose,
    onSave,
    parentMarketName = "market"
}: Props) {
    const { data: user, isLoading: authLoading } = useUserInfo();
    const [formData, setFormData] = useState<ChildMarketData>({
        name: "",
        question: "",
        logo: [],
        liquidity: undefined,
        market_expiry: "",
        slug: "",
        category: "",
        resolution_criteria: "",
    });

    const [errors, setErrors] = useState<Record<string, string>>({});
    const [uploading, setUploading] = useState(false);

    // Store pending files for upload when submitting
    const [pendingFiles, setPendingFiles] = useState<File[]>([]);
    const [pendingReplaceFile, setPendingReplaceFile] = useState<{ file: File; index: number } | null>(null);
    const [uploadProgress, setUploadProgress] = useState<Record<number, number>>({});

    // File input refs
    const fileInputRefs = {
        add: useRef<HTMLInputElement>(null),
        replace: useRef<HTMLInputElement>(null),
    };

    const [replaceIndex, setReplaceIndex] = useState<number | null>(null);

    useEffect(() => {
        if (isOpen && initialData) {
            const logoArray = Array.isArray(initialData.logo) ? initialData.logo :
                (initialData.logo ? [initialData.logo] : []);
            setFormData({
                ...initialData,
                logo: logoArray,
            });
            // Clear pending files when modal opens with existing data
            setPendingFiles([]);
            setPendingReplaceFile(null);
        } else if (isOpen) {
            setFormData({
                name: "",
                question: "",
                logo: [],
                liquidity: undefined,
                market_expiry: "",
                slug: "",
                category: "",
                resolution_criteria: "",
            });
            setPendingFiles([]);
            setPendingReplaceFile(null);
        }
    }, [isOpen, initialData]);

    if (!isOpen) return null;

    const handleChange = (field: keyof ChildMarketData, value: string | number | string[]) => {
        setFormData((prev) => ({ ...prev, [field]: value }));
        if (errors[field]) {
            setErrors((prev) => {
                const newErrors = { ...prev };
                delete newErrors[field];
                return newErrors;
            });
        }
    };

    // Trigger file input
    const triggerFileInput = (docType: keyof typeof fileInputRefs) => {
        fileInputRefs[docType].current?.click();
    };

    // Upload a single file to Firebase
    const uploadSingleFile = async (file: File, index: number): Promise<string> => {
        const userEmail = user?.email || 'admin';
        const marketName = formData.name || parentMarketName;

        setUploadProgress(prev => ({ ...prev, [index]: 0 }));

        const progressInterval = setInterval(() => {
            setUploadProgress(prev => {
                const current = prev[index] || 0;
                if (current < 90) {
                    return { ...prev, [index]: current + 10 };
                }
                return prev;
            });
        }, 200);

        try {
            const url = await uploadFileToFirebase(
                file,
                undefined,
                userEmail,
                `${parentMarketName}_${marketName}_logo_${Date.now()}_${index}`
            );
            clearInterval(progressInterval);
            setUploadProgress(prev => ({ ...prev, [index]: 100 }));
            return url;
        } catch (error) {
            clearInterval(progressInterval);
            throw error;
        }
    };

    // Handle file selection for adding new images - STORE IN PENDING, NOT UPLOAD
    const handleAddFiles = (event: React.ChangeEvent<HTMLInputElement>) => {
        const files = event.target.files;
        if (!files || files.length === 0) return;

        const validFiles: File[] = [];

        // Validate all files
        for (let i = 0; i < files.length; i++) {
            const file = files[i];

            if (!file.type.startsWith('image/')) {
                alert(`Skipping ${file.name}: Not an image file`);
                continue;
            }

            if (file.size > 5 * 1024 * 1024) {
                alert(`Skipping ${file.name}: File size should be less than 5MB`);
                continue;
            }

            validFiles.push(file);
        }

        if (validFiles.length === 0) return;

        // Check capacity
        const currentCount = formData.logo?.length || 0;
        const pendingCount = pendingFiles.length;
        if (currentCount + pendingCount + validFiles.length > 5) {
            alert(`You can only have up to 5 logos. Current: ${currentCount}, Pending: ${pendingCount}, Selected: ${validFiles.length}`);
            event.target.value = '';
            return;
        }

        // Store files in pending state
        setPendingFiles(prev => [...prev, ...validFiles]);

        // Show success message
        alert(`${validFiles.length} image(s) selected. Click "${isEditing ? 'Update' : 'Add'} Child Market" to upload.`);

        event.target.value = '';
    };

    // Handle file selection for replacing existing image - STORE IN PENDING
    const handleReplaceFile = (event: React.ChangeEvent<HTMLInputElement>) => {
        const file = event.target.files?.[0];
        if (!file || replaceIndex === null) return;

        if (!file.type.startsWith('image/')) {
            alert('Please select an image file');
            return;
        }

        if (file.size > 5 * 1024 * 1024) {
            alert('Image size should be less than 5MB');
            return;
        }

        // Store the replace file in pending
        setPendingReplaceFile({ file, index: replaceIndex });

        alert(`Image selected. Click "${isEditing ? 'Update' : 'Add'} Child Market" to upload and replace.`);

        setReplaceIndex(null);
        event.target.value = '';
    };

    // Upload all pending files when submitting
    const uploadPendingFiles = async () => {
        const uploadedUrls: string[] = [];
        const currentLogos = [...(formData.logo || [])];

        // Upload new files
        for (let i = 0; i < pendingFiles.length; i++) {
            const file = pendingFiles[i];
            const newIndex = currentLogos.length + i;

            try {
                const url = await uploadSingleFile(file, newIndex);
                uploadedUrls.push(url);
            } catch (error) {
                console.error(`Failed to upload ${file.name}:`, error);
                throw new Error(`Failed to upload ${file.name}`);
            }
        }

        // Handle replace file if exists
        if (pendingReplaceFile) {
            const { file, index } = pendingReplaceFile;
            try {
                const url = await uploadSingleFile(file, index);
                return { newUrls: uploadedUrls, replaceUrl: { index, url } };
            } catch (error) {
                throw new Error(`Failed to replace image at index ${index}`);
            }
        }

        return { newUrls: uploadedUrls, replaceUrl: null };
    };

    const removeLogo = (index: number) => {
        // If removing a pending file that hasn't been uploaded yet
        const currentLogoCount = formData.logo?.length || 0;
        if (index >= currentLogoCount) {
            const pendingIndex = index - currentLogoCount;
            setPendingFiles(prev => prev.filter((_, i) => i !== pendingIndex));
        } else {
            setFormData((prev) => ({
                ...prev,
                logo: (prev.logo || []).filter((_, i) => i !== index),
            }));
        }
    };

    const openReplacePicker = (index: number) => {
        setReplaceIndex(index);
        triggerFileInput('replace');
    };

    const validateForm = (): boolean => {
        const newErrors: Record<string, string> = {};

        if (!formData.name?.trim()) {
            newErrors.name = "Name is required";
        }
        if (!formData.question?.trim()) {
            newErrors.question = "Question is required";
        }
        if (!formData.liquidity || formData.liquidity <= 0) {
            newErrors.liquidity = "Valid liquidity amount is required";
        }
        if (!formData.market_expiry) {
            newErrors.market_expiry = "Market expiry is required";
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!validateForm()) return;

        // Check if there are pending files to upload
        if (pendingFiles.length === 0 && !pendingReplaceFile) {
            // No files to upload, just save
            const dataToSave: ChildMarketData = {
                ...formData,
                logo: formData.logo || []
            };
            onSave(dataToSave);
            return;
        }

        // Start uploading
        setUploading(true);

        try {
            // Upload all pending files
            const { newUrls, replaceUrl } = await uploadPendingFiles();

            // Update form data with uploaded URLs
            let updatedForm = { ...formData };

            // Add new logos
            if (newUrls.length > 0) {
                updatedForm.logo = [...(formData.logo || []), ...newUrls];
            }

            // Handle replace
            if (replaceUrl) {
                updatedForm.logo = (updatedForm.logo || []).map((url, i) =>
                    i === replaceUrl.index ? replaceUrl.url : url
                );
            }

            // Clear pending files
            setPendingFiles([]);
            setPendingReplaceFile(null);

            // Save with uploaded URLs
            const dataToSave: ChildMarketData = {
                ...updatedForm,
                logo: updatedForm.logo || []
            };
            onSave(dataToSave);

        } catch (error) {
            console.error("Upload error:", error);
            alert(error instanceof Error ? error.message : "Failed to upload images");
        } finally {
            setUploading(false);
            setUploadProgress({});
        }
    };

    // Generate preview URLs for pending files
    const pendingPreviews = pendingFiles.map(file => URL.createObjectURL(file));
    const allLogos = [...(formData.logo || [])];
    const totalItems = allLogos.length + pendingPreviews.length;

    return (
        <>
            {/* Hidden file inputs */}
            <input
                ref={fileInputRefs.add}
                type="file"
                accept="image/*"
                multiple
                style={{ display: 'none' }}
                onChange={handleAddFiles}
            />
            <input
                ref={fileInputRefs.replace}
                type="file"
                accept="image/*"
                style={{ display: 'none' }}
                onChange={handleReplaceFile}
            />

            <Box
                position="fixed"
                inset={0}
                bg="blackAlpha.600"
                display="flex"
                alignItems="center"
                justifyContent="center"
                zIndex={50}
                p={4}
            >
                <Box bg="white" rounded="lg" w="full" maxW="500px" maxH="90vh" overflow="hidden">
                    {/* Header */}
                    <Flex
                        p={4}
                        borderBottom="1px solid"
                        borderColor="gray.200"
                        align="center"
                        justify="space-between"
                        bg="gray.50"
                    >
                        <HStack gap={2}>
                            <Heading size="md">
                                {isEditing ? "Edit" : "Add"} Child Market
                            </Heading>
                            {isEditing && (
                                <Badge colorScheme="blue" fontSize="xs">
                                    Editing
                                </Badge>
                            )}
                        </HStack>
                        <Button
                            onClick={onClose}
                            size="sm"
                            variant="ghost"
                            p={1}
                            _hover={{ bg: "gray.200" }}
                        >
                            <X size={20} />
                        </Button>
                    </Flex>

                    {/* Form */}
                    <Box
                        as="form"
                        onSubmit={handleSubmit}
                        p={4}
                        overflowY="auto"
                        maxH="calc(90vh - 130px)"
                    >
                        <VStack gap={4}>
                            {/* Required Fields Notice */}
                            <Text fontSize="sm" color="gray.500" w="full">
                                <Text as="span" color="red.500">*</Text> Required fields
                            </Text>

                            {/* Name */}
                            <Field.Root required invalid={!!errors.name}>
                                <Field.Label>
                                    Name <Field.RequiredIndicator />
                                </Field.Label>
                                <Input
                                    placeholder="e.g., Will Bitcoin reach $100k?"
                                    value={formData.name || ""}
                                    onChange={(e) => handleChange("name", e.target.value)}
                                />
                                {errors.name && (
                                    <Field.ErrorText>{errors.name}</Field.ErrorText>
                                )}
                            </Field.Root>

                            {/* Question */}
                            <Field.Root required invalid={!!errors.question}>
                                <Field.Label>
                                    Question <Field.RequiredIndicator />
                                </Field.Label>
                                <Input
                                    placeholder="What is the specific question?"
                                    value={formData.question || ""}
                                    onChange={(e) => handleChange("question", e.target.value)}
                                />
                                {errors.question && (
                                    <Field.ErrorText>{errors.question}</Field.ErrorText>
                                )}
                            </Field.Root>

                            {/* Logo Section */}
                            <Field.Root>
                                <Field.Label>Images (from phone gallery)</Field.Label>
                                <VStack align="stretch" gap={3}>
                                    {totalItems > 0 && (
                                        <SimpleGrid columns={{ base: 2, md: 3 }} gap={3}>
                                            {/* Existing uploaded logos */}
                                            {allLogos.map((url, idx) => (
                                                <Box key={`uploaded-${idx}`} position="relative" borderWidth="1px" borderRadius="md" p={2}>
                                                    <img
                                                        src={url}
                                                        alt={`Logo ${idx + 1}`}
                                                        style={{
                                                            height: '100px',
                                                            width: '100%',
                                                            objectFit: 'cover',
                                                            borderRadius: '8px'
                                                        }}
                                                        onError={(e) => {
                                                            e.currentTarget.src = "https://via.placeholder.com/100?text=Logo";
                                                        }}
                                                    />
                                                    {uploadProgress[idx] && uploadProgress[idx] < 100 && (
                                                        <Box
                                                            position="absolute"
                                                            top="50%"
                                                            left="50%"
                                                            transform="translate(-50%, -50%)"
                                                            bg="blackAlpha.700"
                                                            p={2}
                                                            borderRadius="md"
                                                            textAlign="center"
                                                        >
                                                            <Spinner size="sm" />
                                                            <Text fontSize="xs" color="white" mt={1}>
                                                                {uploadProgress[idx]}%
                                                            </Text>
                                                        </Box>
                                                    )}
                                                    <IconButton
                                                        aria-label="Remove image"
                                                        size="xs"
                                                        position="absolute"
                                                        top="0"
                                                        right="0"
                                                        onClick={() => removeLogo(idx)}
                                                        bg="red.500"
                                                        color="white"
                                                        _hover={{ bg: "red.600" }}
                                                    >
                                                        <X size={12} />
                                                    </IconButton>
                                                    <Button
                                                        size="xs"
                                                        variant="ghost"
                                                        mt={2}
                                                        onClick={() => openReplacePicker(idx)}
                                                        loading={uploading}
                                                    >
                                                        Change
                                                    </Button>
                                                </Box>
                                            ))}

                                            {/* Pending files preview */}
                                            {pendingPreviews.map((preview, idx) => {
                                                const actualIndex = allLogos.length + idx;
                                                return (
                                                    <Box key={`pending-${idx}`} position="relative" borderWidth="1px" borderRadius="md" p={2} borderColor="blue.300" bg="blue.50">
                                                        <img
                                                            src={preview}
                                                            alt={`Pending ${idx + 1}`}
                                                            style={{
                                                                height: '100px',
                                                                width: '100%',
                                                                objectFit: 'cover',
                                                                borderRadius: '8px'
                                                            }}
                                                        />
                                                        <Badge position="absolute" top="0" left="0" m={1} size="xs" colorScheme="blue">
                                                            Pending
                                                        </Badge>
                                                        <IconButton
                                                            aria-label="Remove pending"
                                                            size="xs"
                                                            position="absolute"
                                                            top="0"
                                                            right="0"
                                                            onClick={() => removeLogo(actualIndex)}
                                                            bg="red.500"
                                                            color="white"
                                                            _hover={{ bg: "red.600" }}
                                                        >
                                                            <X size={12} />
                                                        </IconButton>
                                                        <Text fontSize="xs" textAlign="center" mt={1} color="blue.600">
                                                            Ready to upload
                                                        </Text>
                                                    </Box>
                                                );
                                            })}
                                        </SimpleGrid>
                                    )}

                                    {totalItems < 5 && (
                                        <Button
                                            size="sm"
                                            variant="outline"
                                            onClick={() => triggerFileInput('add')}
                                            loading={uploading}
                                        >
                                            <Upload size={16} />
                                            <Text ml={2}>Choose from Gallery</Text>
                                        </Button>
                                    )}
                                    {totalItems === 0 && !uploading && (
                                        <Text fontSize="sm" color="gray.500">
                                            Tap to select images from your phone gallery (you can select multiple at once)
                                        </Text>
                                    )}
                                    {pendingFiles.length > 0 && !uploading && (
                                        <Text fontSize="sm" color="blue.500">
                                            {pendingFiles.length} image(s) ready to upload. Click "{isEditing ? 'Update' : 'Add'} Child Market" to upload.
                                        </Text>
                                    )}
                                </VStack>
                            </Field.Root>

                            {/* Slug */}
                            <Field.Root>
                                <Field.Label>Slug (URL-friendly identifier)</Field.Label>
                                <Input
                                    placeholder="bitcoin-100k"
                                    value={formData.slug || ""}
                                    onChange={(e) => handleChange("slug", e.target.value)}
                                />
                            </Field.Root>

                            {/* Category */}
                            <Field.Root>
                                <Field.Label>Category</Field.Label>
                                <NativeSelect.Root>
                                    <NativeSelect.Field
                                        value={formData.category || ""}
                                        onChange={(e) => handleChange("category", e.target.value)}
                                    >
                                        <option value="">Select a category</option>
                                        {CATEGORIES.map((category) => (
                                            <option key={category} value={category}>
                                                {category}
                                            </option>
                                        ))}
                                    </NativeSelect.Field>
                                    <NativeSelect.Indicator />
                                </NativeSelect.Root>
                            </Field.Root>

                            {/* Resolution Criteria */}
                            <Field.Root>
                                <Field.Label>Resolution Criteria</Field.Label>
                                <Textarea
                                    placeholder="How will this market be resolved?"
                                    value={formData.resolution_criteria || ""}
                                    onChange={(e) => handleChange("resolution_criteria", e.target.value)}
                                    rows={3}
                                />
                            </Field.Root>

                            {/* Liquidity */}
                            <Field.Root required invalid={!!errors.liquidity}>
                                <Field.Label>
                                    Liquidity <Field.RequiredIndicator />
                                </Field.Label>
                                <Input
                                    type="number"
                                    step="0.01"
                                    min="0"
                                    placeholder="1000"
                                    value={formData.liquidity || ""}
                                    onChange={(e) => handleChange("liquidity", Number(e.target.value))}
                                />
                                {errors.liquidity && (
                                    <Field.ErrorText>{errors.liquidity}</Field.ErrorText>
                                )}
                            </Field.Root>

                            {/* Market Expiry */}
                            <Field.Root required invalid={!!errors.market_expiry}>
                                <Field.Label>
                                    Market Expiry <Field.RequiredIndicator />
                                </Field.Label>
                                <Input
                                    type="datetime-local"
                                    value={formData.market_expiry || ""}
                                    onChange={(e) => handleChange("market_expiry", e.target.value)}
                                />
                                {errors.market_expiry && (
                                    <Field.ErrorText>{errors.market_expiry}</Field.ErrorText>
                                )}
                            </Field.Root>
                        </VStack>
                    </Box>

                    {/* Footer */}
                    <Flex
                        p={4}
                        borderTop="1px solid"
                        borderColor="gray.200"
                        justify="flex-end"
                        gap={2}
                        bg="gray.50"
                    >
                        <Button variant="outline" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button
                            colorScheme="blue"
                            onClick={handleSubmit}
                            type="submit"
                            loading={uploading}
                            loadingText={pendingFiles.length > 0 ? `Uploading ${pendingFiles.length} images...` : "Saving..."}
                        >
                            {pendingFiles.length > 0
                                ? `${isEditing ? "Update" : "Add"} (${pendingFiles.length} images to upload)`
                                : (isEditing ? "Update" : "Add")} Child Market
                        </Button>
                    </Flex>
                </Box>
            </Box>
        </>
    );
}











// // components/ChildMarketModal.tsx
// import React, { useState, useEffect } from 'react';
// import { X } from 'lucide-react';
// import {
//     Box,
//     Flex,
//     Heading,
//     Text,
//     VStack,
//     HStack,
//     Button,
//     Input,
//     Textarea,
//     Field,
//     NativeSelect,
//     Badge,
// } from "@chakra-ui/react";

// // types/market.ts or at the top of your admin page
// export interface ChildMarketData {
//     name?: string;
//     question?: string;
//     logo?: string;
//     liquidity?: number;
//     market_expiry?: string;
//     slug?: string;
//     category?: string;
//     resolution_criteria?: string;
// }

// interface Props {
//     isOpen: boolean;
//     isEditing: boolean;
//     initialData?: ChildMarketData;
//     onClose: () => void;
//     onSave: (data: ChildMarketData) => void;
// }

// // Predefined categories
// const CATEGORIES = [
//     "Politics",
//     "Sports",
//     "Crypto",
//     "Economics",
//     "Technology",
//     "Entertainment",
//     "Science",
//     "World News",
//     "Business",
//     "Other",
// ];

// export default function ChildMarketModal({
//     isOpen,
//     isEditing,
//     initialData,
//     onClose,
//     onSave
// }: Props) {
//     const [formData, setFormData] = useState<ChildMarketData>({
//         name: "",
//         question: "",
//         logo: "",
//         liquidity: undefined,
//         market_expiry: "",
//         slug: "",
//         category: "",
//         resolution_criteria: "",
//     });

//     const [errors, setErrors] = useState<Record<string, string>>({});

//     // Reset form when modal opens with initial data
//     useEffect(() => {
//         if (isOpen && initialData) {
//             setFormData(initialData);
//         } else if (isOpen) {
//             setFormData({
//                 name: "",
//                 question: "",
//                 logo: "",
//                 liquidity: undefined,
//                 market_expiry: "",
//                 slug: "",
//                 category: "",
//                 resolution_criteria: "",
//             });
//         }
//     }, [isOpen, initialData]);

//     if (!isOpen) return null;

//     const handleChange = (field: keyof ChildMarketData, value: string | number) => {
//         setFormData((prev) => ({ ...prev, [field]: value }));
//         // Clear error for this field if it exists
//         if (errors[field]) {
//             setErrors((prev) => {
//                 const newErrors = { ...prev };
//                 delete newErrors[field];
//                 return newErrors;
//             });
//         }
//     };

//     const validateForm = (): boolean => {
//         const newErrors: Record<string, string> = {};

//         if (!formData.name?.trim()) {
//             newErrors.name = "Name is required";
//         }
//         if (!formData.question?.trim()) {
//             newErrors.question = "Question is required";
//         }
//         if (!formData.liquidity || formData.liquidity <= 0) {
//             newErrors.liquidity = "Valid liquidity amount is required";
//         }
//         if (!formData.market_expiry) {
//             newErrors.market_expiry = "Market expiry is required";
//         }

//         setErrors(newErrors);
//         return Object.keys(newErrors).length === 0;
//     };

//     const handleSubmit = (e: React.FormEvent) => {
//         e.preventDefault();

//         if (validateForm()) {
//             onSave(formData);
//         }
//     };

//     return (
//         <Box
//             position="fixed"
//             inset={0}
//             bg="blackAlpha.600"
//             display="flex"
//             alignItems="center"
//             justifyContent="center"
//             zIndex={50}
//             p={4}
//         >
//             <Box bg="white" rounded="lg" w="full" maxW="500px" maxH="90vh" overflow="hidden">
//                 {/* Header */}
//                 <Flex
//                     p={4}
//                     borderBottom="1px solid"
//                     borderColor="gray.200"
//                     align="center"
//                     justify="space-between"
//                     bg="gray.50"
//                 >
//                     <HStack gap={2}>
//                         <Heading size="md">
//                             {isEditing ? "Edit" : "Add"} Child Market
//                         </Heading>
//                         {isEditing && (
//                             <Badge colorScheme="blue" fontSize="xs">
//                                 Editing
//                             </Badge>
//                         )}
//                     </HStack>
//                     <Button
//                         onClick={onClose}
//                         size="sm"
//                         variant="ghost"
//                         p={1}
//                         _hover={{ bg: "gray.200" }}
//                     >
//                         <X size={20} />
//                     </Button>
//                 </Flex>

//                 {/* Form */}
//                 <Box
//                     as="form"
//                     onSubmit={handleSubmit}
//                     p={4}
//                     overflowY="auto"
//                     maxH="calc(90vh - 130px)"
//                 >
//                     <VStack gap={4}>
//                         {/* Required Fields Notice */}
//                         <Text fontSize="sm" color="gray.500" w="full">
//                             <Text as="span" color="red.500">*</Text> Required fields
//                         </Text>

//                         {/* Name */}
//                         <Field.Root required invalid={!!errors.name}>
//                             <Field.Label>
//                                 Name <Field.RequiredIndicator />
//                             </Field.Label>
//                             <Input
//                                 placeholder="e.g., Will Bitcoin reach $100k?"
//                                 value={formData.name || ""}
//                                 onChange={(e) => handleChange("name", e.target.value)}
//                             />
//                             {errors.name && (
//                                 <Field.ErrorText>{errors.name}</Field.ErrorText>
//                             )}
//                         </Field.Root>

//                         {/* Question */}
//                         <Field.Root required invalid={!!errors.question}>
//                             <Field.Label>
//                                 Question <Field.RequiredIndicator />
//                             </Field.Label>
//                             <Input
//                                 placeholder="What is the specific question?"
//                                 value={formData.question || ""}
//                                 onChange={(e) => handleChange("question", e.target.value)}
//                             />
//                             {errors.question && (
//                                 <Field.ErrorText>{errors.question}</Field.ErrorText>
//                             )}
//                         </Field.Root>

//                         {/* Logo URL */}
//                         <Field.Root>
//                             <Field.Label>Logo URL</Field.Label>
//                             <Input
//                                 placeholder="https://example.com/logo.png"
//                                 value={formData.logo || ""}
//                                 onChange={(e) => handleChange("logo", e.target.value)}
//                             />
//                         </Field.Root>

//                         {/* Slug */}
//                         <Field.Root>
//                             <Field.Label>Slug (URL-friendly identifier)</Field.Label>
//                             <Input
//                                 placeholder="bitcoin-100k"
//                                 value={formData.slug || ""}
//                                 onChange={(e) => handleChange("slug", e.target.value)}
//                             />
//                         </Field.Root>

//                         {/* Category */}
//                         <Field.Root>
//                             <Field.Label>Category</Field.Label>
//                             <NativeSelect.Root>
//                                 <NativeSelect.Field
//                                     value={formData.category || ""}
//                                     onChange={(e) => handleChange("category", e.target.value)}
//                                 >
//                                     <option value="">Select a category</option>
//                                     {CATEGORIES.map((category) => (
//                                         <option key={category} value={category}>
//                                             {category}
//                                         </option>
//                                     ))}
//                                 </NativeSelect.Field>
//                                 <NativeSelect.Indicator />
//                             </NativeSelect.Root>
//                         </Field.Root>

//                         {/* Resolution Criteria */}
//                         <Field.Root>
//                             <Field.Label>Resolution Criteria</Field.Label>
//                             <Textarea
//                                 placeholder="How will this market be resolved? e.g., Official announcement, Oracle data, etc."
//                                 value={formData.resolution_criteria || ""}
//                                 onChange={(e) => handleChange("resolution_criteria", e.target.value)}
//                                 rows={3}
//                             />
//                         </Field.Root>

//                         {/* Liquidity */}
//                         <Field.Root required invalid={!!errors.liquidity}>
//                             <Field.Label>
//                                 Liquidity <Field.RequiredIndicator />
//                             </Field.Label>
//                             <Input
//                                 type="number"
//                                 step="0.01"
//                                 min="0"
//                                 placeholder="1000"
//                                 value={formData.liquidity || ""}
//                                 onChange={(e) => handleChange("liquidity", Number(e.target.value))}
//                             />
//                             {errors.liquidity && (
//                                 <Field.ErrorText>{errors.liquidity}</Field.ErrorText>
//                             )}
//                         </Field.Root>

//                         {/* Market Expiry */}
//                         <Field.Root required invalid={!!errors.market_expiry}>
//                             <Field.Label>
//                                 Market Expiry <Field.RequiredIndicator />
//                             </Field.Label>
//                             <Input
//                                 type="datetime-local"
//                                 value={formData.market_expiry || ""}
//                                 onChange={(e) => handleChange("market_expiry", e.target.value)}
//                             />
//                             {errors.market_expiry && (
//                                 <Field.ErrorText>{errors.market_expiry}</Field.ErrorText>
//                             )}
//                         </Field.Root>
//                     </VStack>
//                 </Box>

//                 {/* Footer */}
//                 <Flex
//                     p={4}
//                     borderTop="1px solid"
//                     borderColor="gray.200"
//                     justify="flex-end"
//                     gap={2}
//                     bg="gray.50"
//                 >
//                     <Button variant="outline" onClick={onClose}>
//                         Cancel
//                     </Button>
//                     <Button
//                         colorScheme="blue"
//                         onClick={handleSubmit}
//                         type="submit"
//                     >
//                         {isEditing ? "Update" : "Add"} Child Market
//                     </Button>
//                 </Flex>
//             </Box>
//         </Box>
//     );
// }




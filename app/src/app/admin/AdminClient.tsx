"use client";

import {
    Box,
    Text,
    Input,
    Textarea,
    Button,
    VStack,
    HStack,
    IconButton,
    Badge,
    Card,
    CardBody,
    Switch,
    Field,
    NativeSelect,
    SimpleGrid,
    Spinner,
} from "@chakra-ui/react";
import { useState, useRef } from "react";
import { useMutation } from "@tanstack/react-query";
import { toaster } from "@/components/ui/toaster";

import { Plus, Trash2, Edit, ImageIcon, X, Upload } from "lucide-react";
import { MarketActions } from "@/utils/interactions/dataPosters";

import ChildMarketModal, { ChildMarketData } from "@/components/ChildMarketModal";
import { useAuth } from "@/context/AuthContext";
import { uploadFileToFirebase } from "../lib/services/firebase";
import useUserInfo from "@/hooks/useUserInfo";

interface CreateMarketRequest {
    name?: string;
    description?: string;
    logo?: string[];
    liquidity_b?: number;
    market_expiry?: string;
    slug?: string;
    is_event?: boolean;
    child_markets?: CreateChildMarketRequest[];
    category?: string;
    resolution_criteria?: string;
}

interface CreateChildMarketRequest {
    name?: string;
    question?: string;
    logo?: string[];
    liquidity?: number;
    market_expiry?: string;
    slug?: string;
    category?: string;
    resolution_criteria?: string;
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

const AdminDashboardPage = () => {
    console.log("API KEY:", process.env.NEXT_PUBLIC_FIREBASE_API_KEY);

    const { data: user, isLoading: authLoading } = useUserInfo();
    const [form, setForm] = useState<CreateMarketRequest>({
        name: "",
        description: "",
        logo: [],
        liquidity_b: undefined,
        market_expiry: "",
        slug: "",
        is_event: false,
        child_markets: [],
        category: "",
        resolution_criteria: "",
    });

    const [childModalOpen, setChildModalOpen] = useState(false);
    const [editingChild, setEditingChild] = useState<CreateChildMarketRequest | undefined>();
    const [editingChildIndex, setEditingChildIndex] = useState<number | null>(null);
    const [childForm, setChildForm] = useState<CreateChildMarketRequest>({
        name: "",
        question: "",
        logo: [],
        liquidity: undefined,
        market_expiry: "",
        slug: "",
        category: "",
        resolution_criteria: "",
    });

    // File input refs for parent logos
    const fileInputRefs = {
        parent: useRef<HTMLInputElement>(null),
        parentReplace: useRef<HTMLInputElement>(null),
    };

    // Store pending files for upload when submitting
    const [pendingFiles, setPendingFiles] = useState<File[]>([]);
    const [pendingReplaceFile, setPendingReplaceFile] = useState<{ file: File; index: number } | null>(null);
    const [uploading, setUploading] = useState(false);
    const [uploadProgress, setUploadProgress] = useState<Record<number, number>>({});

    const mutation = useMutation({
        mutationFn: async (data: CreateMarketRequest) => {
            return await MarketActions.createMarket(data);
        },
    });

    const { isPending } = mutation;

    const handleChange = (field: keyof CreateMarketRequest, value: string | number | boolean) => {
        setForm((prev) => ({ ...prev, [field]: value }));
    };

    // Trigger file picker
    const triggerFileInput = (type: 'add' | 'replace') => {
        if (type === 'add') {
            fileInputRefs.parent.current?.click();
        } else {
            fileInputRefs.parentReplace.current?.click();
        }
    };

    // Handle file selection for adding new logos - STORE IN PENDING, NOT UPLOAD
    const handleAddFiles = async (event: React.ChangeEvent<HTMLInputElement>) => {
        const files = event.target.files;
        if (!files || files.length === 0) return;

        const validFiles: File[] = [];

        // Validate all selected files
        for (let i = 0; i < files.length; i++) {
            const file = files[i];

            if (!file.type.startsWith('image/')) {
                toaster.warning({
                    title: "Skipped",
                    description: `${file.name} is not an image file`,
                });
                continue;
            }

            if (file.size > 5 * 1024 * 1024) {
                toaster.warning({
                    title: "Skipped",
                    description: `${file.name} exceeds 5MB limit`,
                });
                continue;
            }

            validFiles.push(file);
        }

        if (validFiles.length === 0) return;

        // Check capacity
        const currentCount = form.logo?.length || 0;
        const pendingCount = pendingFiles.length;
        if (currentCount + pendingCount + validFiles.length > 5) {
            toaster.error({
                title: "Too many images",
                description: `You can only have up to 5 logos. Current: ${currentCount}, Pending: ${pendingCount}, Selected: ${validFiles.length}`,
            });
            event.target.value = '';
            return;
        }

        // Store files in pending state
        setPendingFiles(prev => [...prev, ...validFiles]);

        // Show preview of pending files
        toaster.success({
            title: "Files selected",
            description: `${validFiles.length} image(s) ready to upload. Click "Create Market" to upload.`,
        });

        event.target.value = '';
    };

    // Handle file selection for replacing existing logo - STORE IN PENDING
    const handleReplaceFile = async (event: React.ChangeEvent<HTMLInputElement>) => {
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

        toaster.success({
            title: "File selected",
            description: "Image will be uploaded when you click 'Create Market'",
        });

        setReplaceIndex(null);
        event.target.value = '';
    };

    // Upload a single file to Firebase
    const uploadSingleFile = async (file: File, index: number): Promise<string> => {
        const userEmail = user?.email;
        if (!userEmail) throw new Error("User email not found");

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
                `market_logo_${Date.now()}_${index}`
            );
            clearInterval(progressInterval);
            setUploadProgress(prev => ({ ...prev, [index]: 100 }));
            return url;
        } catch (error) {
            clearInterval(progressInterval);
            throw error;
        }
    };

    // Upload all pending files when submitting
    const uploadPendingFiles = async () => {
        const uploadedUrls: string[] = [];
        const currentLogos = [...(form.logo || [])];

        // Upload new files first
        for (let i = 0; i < pendingFiles.length; i++) {
            const file = pendingFiles[i];
            const newIndex = currentLogos.length + i;

            try {
                const url = await uploadSingleFile(file, newIndex);
                uploadedUrls.push(url);

                toaster.success({
                    title: `Uploaded ${i + 1}/${pendingFiles.length}`,
                    description: `${file.name} uploaded successfully`,
                });
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

    const addLogo = () => {
        triggerFileInput('add');
    };

    const removeLogo = (index: number) => {
        // If removing a pending file that hasn't been uploaded yet
        if (index >= (form.logo?.length || 0)) {
            const pendingIndex = index - (form.logo?.length || 0);
            setPendingFiles(prev => prev.filter((_, i) => i !== pendingIndex));
        } else {
            setForm((prev) => ({
                ...prev,
                logo: (prev.logo || []).filter((_, i) => i !== index),
            }));
        }
    };

    const [replaceIndex, setReplaceIndex] = useState<number | null>(null);

    const openReplacePicker = (index: number) => {
        setReplaceIndex(index);
        triggerFileInput('replace');
    };

    const resetChildForm = () => {
        setChildForm({
            name: "",
            question: "",
            logo: [],
            liquidity: undefined,
            market_expiry: "",
            slug: "",
            category: "",
            resolution_criteria: "",
        });
        setEditingChildIndex(null);
    };

    const openChildDialog = (index?: number) => {
        if (index !== undefined) {
            const child = form.child_markets?.[index];
            if (child) {
                setEditingChild(child);
                setEditingChildIndex(index);
            }
        } else {
            setEditingChild(undefined);
            setEditingChildIndex(null);
        }
        setChildModalOpen(true);
    };

    const saveChildMarket = (data: ChildMarketData) => {
        if (!data.name || !data.question || !data.liquidity || !data.market_expiry) {
            toaster.error({
                title: "Validation Error",
                description: "Name, question, liquidity, and expiry are required for child markets",
            });
            return;
        }

        setForm((prev) => {
            const updatedChildren = [...(prev.child_markets || [])];

            const logoArray = Array.isArray(data.logo) ? data.logo :
                (data.logo ? [data.logo] : []);

            const childData = {
                name: data.name,
                question: data.question,
                logo: logoArray,
                liquidity: data.liquidity,
                market_expiry: data.market_expiry,
                slug: data.slug,
                category: data.category,
                resolution_criteria: data.resolution_criteria,
            };

            if (editingChildIndex !== null) {
                updatedChildren[editingChildIndex] = childData;
            } else {
                updatedChildren.push(childData);
            }

            return {
                ...prev,
                child_markets: updatedChildren,
            };
        });

        setChildModalOpen(false);
        setEditingChild(undefined);
        setEditingChildIndex(null);
    };

    const removeChildMarket = (index: number) => {
        setForm((prev) => ({
            ...prev,
            child_markets: prev.child_markets?.filter((_, i) => i !== index) || [],
        }));
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        // Validate form
        if (!form.name || !form.description || !form.liquidity_b || !form.market_expiry) {
            toaster.error({
                title: "Validation Error",
                description: "Please fill in all required fields",
            });
            return;
        }

        // Check if there are pending files to upload
        if (pendingFiles.length === 0 && !pendingReplaceFile) {
            // No files to upload, just submit
            await submitMarket(form);
            return;
        }

        // Start uploading
        setUploading(true);

        try {
            // Upload all pending files
            const { newUrls, replaceUrl } = await uploadPendingFiles();

            // Update form with uploaded URLs
            let updatedForm = { ...form };

            // Add new logos
            if (newUrls.length > 0) {
                updatedForm.logo = [...(form.logo || []), ...newUrls];
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

            // Submit the market with uploaded URLs
            await submitMarket(updatedForm);

        } catch (error) {
            console.error("Upload error:", error);
            toaster.error({
                title: "Upload Failed",
                description: error instanceof Error ? error.message : "Failed to upload images",
            });
        } finally {
            setUploading(false);
            setUploadProgress({});
        }
    };

    const submitMarket = async (marketData: CreateMarketRequest) => {
        const generateUniqueSlug = (name: string, suffix: string = ''): string => {
            const baseSlug = name
                .toLowerCase()
                .replace(/[^a-z0-9]+/g, '-')
                .replace(/^-|-$/g, '')
                .substring(0, 50);

            const uniqueId = `${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;
            return `${baseSlug}-${uniqueId}${suffix}`;
        };

        const parentSlug = marketData.slug || generateUniqueSlug(marketData.name || '');

        const payload: CreateMarketRequest = {
            name: marketData.name,
            description: marketData.description,
            logo: marketData.logo && marketData.logo.length > 0 ? marketData.logo : undefined,
            liquidity_b: marketData.liquidity_b,
            market_expiry: marketData.market_expiry,
            slug: parentSlug,
            is_event: marketData.is_event || false,
            category: marketData.category || undefined,
            resolution_criteria: marketData.resolution_criteria || undefined,
            child_markets: marketData.child_markets?.map((child, index) => {
                const childSlug = child.slug || generateUniqueSlug(child.name || 'child', `-${index}`);
                return {
                    name: child.name,
                    question: child.question,
                    logo: child.logo && child.logo.length > 0 ? child.logo : undefined,
                    liquidity: child.liquidity,
                    market_expiry: child.market_expiry,
                    slug: childSlug,
                    category: child.category || undefined,
                    resolution_criteria: child.resolution_criteria || undefined,
                };
            }),
        };

        const allSlugs = [
            payload.slug,
            ...(payload.child_markets?.map(c => c.slug) || [])
        ].filter(Boolean);

        const uniqueSlugs = new Set(allSlugs);
        if (uniqueSlugs.size !== allSlugs.length) {
            console.error("❌ DUPLICATE SLUGS DETECTED!", allSlugs);
            toaster.error({
                title: "Duplicate Slugs",
                description: "Please check your child markets for duplicate slugs",
            });
            return;
        }

        toaster.promise(mutation.mutateAsync(payload), {
            loading: { title: "Creating market..." },
            success: () => ({ title: "Market created successfully!" }),
            error: (e: any) => ({
                title: "Failed",
                description: e?.message || "Check console for details"
            }),
        });
    };

    // Logo Display Component with preview for pending files
    const LogoSection = ({
        logos,
        pendingFiles,
        onAdd,
        onRemove,
        onReplace,
        label,
        uploading
    }: {
        logos: string[];
        pendingFiles: File[];
        onAdd: () => void;
        onRemove: (index: number) => void;
        onReplace: (index: number) => void;
        label: string;
        uploading: boolean;
    }) => {
        const allLogos = [...logos];
        const pendingPreviews = pendingFiles.map(file => URL.createObjectURL(file));
        const totalItems = allLogos.length + pendingPreviews.length;

        return (
            <Field.Root>
                <Field.Label>{label} Logos (up to 5)</Field.Label>
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
                                    <IconButton
                                        aria-label="Remove logo"
                                        size="xs"
                                        position="absolute"
                                        top="0"
                                        right="0"
                                        onClick={() => onRemove(idx)}
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
                                        onClick={() => onReplace(idx)}
                                        loading={uploading}
                                    >
                                        Change
                                    </Button>
                                </Box>
                            ))}

                            {/* Pending files preview */}
                            {pendingPreviews.map((preview, idx) => (
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
                                        onClick={() => onRemove(allLogos.length + idx)}
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
                            ))}
                        </SimpleGrid>
                    )}

                    {totalItems < 5 && (
                        <HStack gap={2}>
                            <Button size="sm" variant="outline" onClick={onAdd} loading={uploading}>
                                <Plus size={16} />
                                <Text ml={1}>Add URL</Text>
                            </Button>
                            <Button size="sm" variant="outline" onClick={onAdd} loading={uploading}>
                                <Upload size={16} />
                                <Text ml={1}>Choose from Gallery</Text>
                            </Button>
                        </HStack>
                    )}
                    {totalItems === 0 && !uploading && (
                        <Text fontSize="sm" color="gray.500">No logos added. Add at least one logo for your market.</Text>
                    )}
                    {pendingFiles.length > 0 && (
                        <Text fontSize="sm" color="blue.500">
                            {pendingFiles.length} image(s) ready to upload. Click "Create Market" to upload.
                        </Text>
                    )}
                </VStack>
            </Field.Root>
        );
    };

    if (authLoading) {
        return (
            <Box p={6} textAlign="center">
                <Spinner size="xl" />
                <Text mt={4}>Loading user data...</Text>
            </Box>
        );
    }

    return (
        <Box p={6} maxW="800px" mx="auto">
            {/* Hidden file inputs - multiple enabled for add */}
            <input
                ref={fileInputRefs.parent}
                type="file"
                accept="image/*"
                multiple
                style={{ display: 'none' }}
                onChange={handleAddFiles}
            />
            <input
                ref={fileInputRefs.parentReplace}
                type="file"
                accept="image/*"
                style={{ display: 'none' }}
                onChange={handleReplaceFile}
            />

            <Text fontSize="2xl" fontWeight="bold" mb={4}>
                Create a Market
            </Text>

            <form onSubmit={handleSubmit}>
                <VStack gap={4} align="stretch">
                    <Field.Root required>
                        <Field.Label>Market Name</Field.Label>
                        <Input
                            placeholder="Enter market name"
                            value={form.name || ""}
                            onChange={(e) => handleChange("name", e.target.value)}
                        />
                    </Field.Root>

                    <Field.Root required>
                        <Field.Label>Description</Field.Label>
                        <Textarea
                            placeholder="Enter market description"
                            value={form.description || ""}
                            onChange={(e) => handleChange("description", e.target.value)}
                        />
                    </Field.Root>

                    <LogoSection
                        logos={form.logo || []}
                        pendingFiles={pendingFiles}
                        onAdd={addLogo}
                        onRemove={removeLogo}
                        onReplace={openReplacePicker}
                        label="Market"
                        uploading={uploading}
                    />

                    <Field.Root>
                        <Field.Label>Slug (URL-friendly identifier)</Field.Label>
                        <Input
                            placeholder="e.g., my-market-name"
                            value={form.slug || ""}
                            onChange={(e) => handleChange("slug", e.target.value)}
                        />
                    </Field.Root>

                    <Field.Root>
                        <Field.Label>Category</Field.Label>
                        <NativeSelect.Root>
                            <NativeSelect.Field
                                value={form.category || ""}
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

                    <Field.Root>
                        <Field.Label>Resolution Criteria</Field.Label>
                        <Textarea
                            placeholder="How will this market be resolved?"
                            value={form.resolution_criteria || ""}
                            onChange={(e) => handleChange("resolution_criteria", e.target.value)}
                        />
                    </Field.Root>

                    <Field.Root required>
                        <Field.Label>Liquidity (B)</Field.Label>
                        <Input
                            type="number"
                            placeholder="Enter liquidity"
                            value={form.liquidity_b || ""}
                            onChange={(e) => handleChange("liquidity_b", Number(e.target.value))}
                        />
                    </Field.Root>

                    <Field.Root required>
                        <Field.Label>Market Expiry</Field.Label>
                        <Input
                            type="datetime-local"
                            value={form.market_expiry || ""}
                            onChange={(e) => handleChange("market_expiry", e.target.value)}
                        />
                    </Field.Root>

                    <Box p={4} bg="gray.50" borderRadius="md">
                        <Field.Root>
                            <HStack justify="space-between">
                                <Field.Label>Is this an Event? (Will contain child markets)</Field.Label>
                                <Switch.Root
                                    checked={form.is_event}
                                    onCheckedChange={(e) => handleChange("is_event", e.checked)}
                                >
                                    <Switch.HiddenInput />
                                    <Switch.Control>
                                        <Switch.Thumb />
                                    </Switch.Control>
                                    <Switch.Label />
                                </Switch.Root>
                            </HStack>
                        </Field.Root>
                    </Box>

                    {form.is_event && (
                        <Box borderWidth="1px" borderRadius="md" p={4}>
                            <HStack justify="space-between" mb={4}>
                                <Text fontSize="lg" fontWeight="medium">
                                    Child Markets ({form.child_markets?.length || 0})
                                </Text>
                                <Button
                                    size="sm"
                                    colorScheme="blue"
                                    onClick={() => {
                                        resetChildForm();
                                        setChildModalOpen(true);
                                    }}
                                >
                                    <Plus size={16} style={{ marginRight: '4px' }} />
                                    Add Child Market
                                </Button>
                            </HStack>

                            <VStack gap={3} align="stretch">
                                {form.child_markets?.map((child, index) => (
                                    <Card.Root key={index} size="sm">
                                        <CardBody>
                                            <HStack justify="space-between">
                                                <VStack align="start" gap={1}>
                                                    <Text fontWeight="medium">{child.name}</Text>
                                                    <Text fontSize="sm" color="gray.600">{child.question}</Text>
                                                    <HStack gap={2}>
                                                        <Badge colorScheme="green">
                                                            Liq: {child.liquidity}
                                                        </Badge>
                                                        <Badge colorScheme="blue">
                                                            Exp: {new Date(child.market_expiry || "").toLocaleDateString()}
                                                        </Badge>
                                                        {child.category && (
                                                            <Badge colorScheme="purple">
                                                                {child.category}
                                                            </Badge>
                                                        )}
                                                    </HStack>
                                                </VStack>
                                                <HStack>
                                                    <IconButton
                                                        aria-label="Edit child market"
                                                        size="sm"
                                                        variant="ghost"
                                                        onClick={() => {
                                                            setEditingChild(child);
                                                            setEditingChildIndex(index);
                                                            setChildModalOpen(true);
                                                        }}
                                                    >
                                                        <Edit size={16} />
                                                    </IconButton>
                                                    <IconButton
                                                        aria-label="Remove child market"
                                                        size="sm"
                                                        variant="ghost"
                                                        colorScheme="red"
                                                        onClick={() => removeChildMarket(index)}
                                                    >
                                                        <Trash2 size={16} />
                                                    </IconButton>
                                                </HStack>
                                            </HStack>
                                        </CardBody>
                                    </Card.Root>
                                ))}
                            </VStack>
                        </Box>
                    )}

                    <Button
                        type="submit"
                        colorScheme="blue"
                        mt={4}
                        loading={isPending || uploading}
                        loadingText={pendingFiles.length > 0 ? `Uploading ${pendingFiles.length} images...` : "Creating..."}
                    >
                        {pendingFiles.length > 0 ? `Create Market (${pendingFiles.length} images to upload)` : "Create Market"}
                    </Button>
                </VStack>
            </form>

            <ChildMarketModal
                isOpen={childModalOpen}
                isEditing={editingChildIndex !== null}
                initialData={editingChild}
                onClose={() => {
                    setChildModalOpen(false);
                    setEditingChild(undefined);
                    setEditingChildIndex(null);
                }}
                onSave={saveChildMarket}
                parentMarketName={form.name}
            />
        </Box>
    );
};

export default AdminDashboardPage;

"use client";

import { useState, useEffect } from "react";
import {
    Box,
    Container,
    Heading,
    Text,
    VStack,
    HStack,
    Avatar,
    Button,
    Textarea,
    Input,
    Separator,
    Badge,
    Center,
    Spinner,
    Card,
    CardBody,
    CardHeader,
    CardFooter,
    Menu,
    Dialog,
    Field,

} from "@chakra-ui/react";
import { ArrowUp, Plus, Send, ThumbsUp, MoreVertical, Check, X, Clock, Eye, CheckCircle } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { formatDistanceToNow } from "date-fns";
import { toaster } from "@/components/ui/toaster";
import useUserInfo from "@/hooks/useUserInfo";

import { CreateSuggestionRequest, Suggestion, SuggestionAPII, SuggestionResponse, UpdateStatusRequest } from "@/utils/interactions/dataPosters";
import { useAuth } from "@/context/AuthContext";
import { SuggestionAPI } from "@/utils/interactions/dataGetter";


interface SuggestionsClientProps {
    initialSuggestions?: SuggestionResponse[];
}

// Status badge color mapping
const statusColors = {
    pending: 'gray',
    in_review: 'yellow',
    approved: 'green',
    rejected: 'red',
    implemented: 'purple'
};

export default function SuggestionsClient({
    initialSuggestions = [],
}: SuggestionsClientProps) {
    const { isAdmin } = useAuth(); // Get admin status from auth context
    const [suggestions, setSuggestions] = useState<SuggestionResponse[]>(initialSuggestions);
    const { data: userInfo, isLoading: userLoading } = useUserInfo();
    const [showCreateForm, setShowCreateForm] = useState(false);

    // Admin dialog state
    const [selectedSuggestion, setSelectedSuggestion] = useState<SuggestionResponse | null>(null);
    const [adminNotes, setAdminNotes] = useState("");
    const [selectedStatus, setSelectedStatus] = useState<UpdateStatusRequest['status']>('pending');
    const [isAdminDialogOpen, setIsAdminDialogOpen] = useState(false);

    // Create form state
    const [title, setTitle] = useState("");
    const [description, setDescription] = useState("");
    const [category, setCategory] = useState("");

    const queryClient = useQueryClient();

    // Fetch suggestions if not provided
    useEffect(() => {
        if (initialSuggestions.length === 0) {
            const fetchSuggestions = async () => {
                try {
                    let data;
                    if (userInfo) {
                        // Pass admin status to get appropriate data
                        data = await SuggestionAPI.getSuggestions();
                    } else {
                        const publicData = await SuggestionAPI.getPublicSuggestions();
                        data = publicData.map(item => ({
                            ...item,
                            user_voted: false
                        }));
                    }
                    setSuggestions(data);
                } catch (error) {
                    console.error("Failed to fetch suggestions:", error);
                    toaster.error({
                        title: "Failed to load suggestions",
                    });
                }
            };
            fetchSuggestions();
        }
    }, [initialSuggestions.length, userInfo, isAdmin]);

    // Create suggestion mutation
    const createMutation = useMutation<Suggestion, Error, CreateSuggestionRequest>({
        mutationFn: (data) => SuggestionAPII.createSuggestion(data),
    });

    // Upvote mutation
    const upvoteMutation = useMutation<void, Error, string>({
        mutationFn: (suggestionId) => SuggestionAPII.upvoteSuggestion(suggestionId),
    });

    // Admin update status mutation
    const updateStatusMutation = useMutation<Suggestion, Error, { suggestionId: string; req: UpdateStatusRequest }>({
        mutationFn: ({ suggestionId, req }) =>
            SuggestionAPII.updateSuggestionStatus(suggestionId, req),
    });

    const handleCreateSuggestion = () => {
        if (!userInfo) {
            toaster.info({
                title: "Please login to suggest",
                description: "You need to be logged in to create suggestions.",
            });
            return;
        }

        if (!title.trim() || !description.trim()) {
            toaster.error({
                title: "Missing fields",
                description: "Title and description are required.",
            });
            return;
        }

        toaster.promise(
            createMutation.mutateAsync({
                title,
                description,
                category: category || null,
            }),
            {
                loading: { title: "Creating suggestion..." },
                success: (newSuggestion) => {
                    const newSuggestionResponse: SuggestionResponse = {
                        id: newSuggestion.id,
                        user_name: userInfo.name,
                        user_avatar: userInfo.avatar || null,
                        title: newSuggestion.title,
                        description: newSuggestion.description,
                        category: newSuggestion.category,
                        upvotes: newSuggestion.upvotes,
                        status: newSuggestion.status,
                        admin_notes: newSuggestion.admin_notes,
                        created_at: newSuggestion.created_at,
                        updated_at: newSuggestion.updated_at,
                        user_voted: false,
                    };

                    setSuggestions([newSuggestionResponse, ...suggestions]);
                    setTitle("");
                    setDescription("");
                    setCategory("");
                    setShowCreateForm(false);
                    queryClient.invalidateQueries({ queryKey: ["suggestions"] });
                    return { title: "Suggestion created successfully!" };
                },
                error: (e) => ({
                    title: "Failed to create suggestion",
                    description: e instanceof Error ? e.message : "An error occurred",
                }),
            }
        );
    };

    const handleUpvote = (suggestionId: string) => {
        if (!userInfo) {
            toaster.info({
                title: "Please login to upvote",
                description: "You need to be logged in to upvote suggestions.",
            });
            return;
        }

        toaster.promise(
            upvoteMutation.mutateAsync(suggestionId),
            {
                loading: { title: "Upvoting..." },
                success: () => {
                    setSuggestions(suggestions.map(s =>
                        s.id === suggestionId
                            ? {
                                ...s,
                                upvotes: s.upvotes + 1,
                                user_voted: true
                            }
                            : s
                    ));
                    queryClient.invalidateQueries({ queryKey: ["suggestions"] });
                    return { title: "Upvoted successfully!" };
                },
                error: (e) => ({
                    title: "Failed to upvote",
                    description: e instanceof Error ? e.message : "An error occurred",
                }),
            }
        );
    };

    // Admin: Open status update dialog
    const handleAdminAction = (suggestion: SuggestionResponse) => {
        setSelectedSuggestion(suggestion);
        setSelectedStatus(suggestion.status as UpdateStatusRequest['status']);
        setAdminNotes(suggestion.admin_notes || "");
        setIsAdminDialogOpen(true);
    };

    // Admin: Update suggestion status
    const handleUpdateStatus = () => {
        if (!selectedSuggestion || !isAdmin) return;

        toaster.promise(
            updateStatusMutation.mutateAsync({
                suggestionId: selectedSuggestion.id,
                req: {
                    status: selectedStatus,
                    admin_notes: adminNotes || undefined,
                },
            }),
            {
                loading: { title: "Updating suggestion..." },
                success: (updatedSuggestion) => {
                    // Update local state
                    setSuggestions(suggestions.map(s =>
                        s.id === selectedSuggestion.id
                            ? {
                                ...s,
                                status: updatedSuggestion.status,
                                admin_notes: updatedSuggestion.admin_notes,
                                updated_at: updatedSuggestion.updated_at,
                            }
                            : s
                    ));

                    setIsAdminDialogOpen(false);
                    setSelectedSuggestion(null);
                    setAdminNotes("");
                    setSelectedStatus('pending');

                    queryClient.invalidateQueries({ queryKey: ["suggestions"] });

                    return {
                        title: "Suggestion updated!",
                        description: `Status changed to ${selectedStatus}`
                    };
                },
                error: (e) => ({
                    title: "Failed to update suggestion",
                    description: e instanceof Error ? e.message : "An error occurred",
                }),
            }
        );
    };

    // Get icon for status
    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'approved': return <Check size={14} />;
            case 'rejected': return <X size={14} />;
            case 'in_review': return <Eye size={14} />;
            case 'implemented': return <CheckCircle size={14} />;
            default: return <Clock size={14} />;
        }
    };

    // Show loading state
    if (userLoading) {
        return (
            <Center py={10}>
                <Spinner />
            </Center>
        );
    }

    return (
        <Container maxW="4xl" py={8}>
            {/* Header */}
            <Box mb={8}>
                <Heading size="lg" mb={2}>
                    💡 Market Suggestions
                    {isAdmin && <Badge ml={2} colorScheme="purple">Admin View</Badge>}
                </Heading>
                <Text color="gray.600">
                    {isAdmin
                        ? "Review and manage market suggestions from the community."
                        : "Suggest new markets for the platform. Upvote ideas you like!"}
                </Text>
            </Box>

            {/* Create Suggestion Button - Hide for admin if they don't need it */}
            {!isAdmin && !showCreateForm && (
                <Button
                    colorScheme="blue"
                    onClick={() => setShowCreateForm(true)}
                    mb={6}
                >
                    <HStack gap={2}>
                        <Plus size={16} />
                        <Text>Suggest a Market</Text>
                    </HStack>
                </Button>
            )}

            {/* Create Suggestion Form - Only for non-admins */}
            {!isAdmin && showCreateForm && (
                <Box mb={8} p={4} bg="gray.50" borderRadius="lg">
                    <VStack gap={4}>
                        <Input
                            placeholder="Title (e.g., Will Bitcoin reach $100k?)"
                            value={title}
                            onChange={(e) => setTitle(e.target.value)}
                        />
                        <Textarea
                            placeholder="Detailed description of the market idea..."
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            rows={4}
                        />
                        <Input
                            placeholder="Category (optional, e.g., Crypto, Politics, Sports)"
                            value={category}
                            onChange={(e) => setCategory(e.target.value)}
                        />
                        <HStack gap={2} width="full" justify="flex-end">
                            <Button
                                variant="ghost"
                                onClick={() => {
                                    setShowCreateForm(false);
                                    setTitle("");
                                    setDescription("");
                                    setCategory("");
                                }}
                            >
                                Cancel
                            </Button>
                            <Button
                                colorScheme="blue"
                                onClick={handleCreateSuggestion}
                                loading={createMutation.isPending}
                                disabled={!title.trim() || !description.trim()}
                            >
                                <HStack gap={2}>
                                    <Send size={16} />
                                    <Text>Submit Suggestion</Text>
                                </HStack>
                            </Button>
                        </HStack>
                    </VStack>
                </Box>
            )}

            <Separator mb={6} />

            {/* Suggestions List */}
            <VStack gap={4} align="stretch">
                {suggestions.length === 0 ? (
                    <Center py={10}>
                        <Text color="gray.500">
                            {isAdmin
                                ? "No suggestions to review yet."
                                : "No suggestions yet. Be the first to suggest a market!"}
                        </Text>
                    </Center>
                ) : (
                    suggestions.map((suggestion) => (
                        <Card.Root key={suggestion.id} variant="outline">
                            <CardHeader>
                                <HStack justify="space-between">
                                    <HStack gap={2}>
                                        <Avatar.Root size="sm">
                                            <Avatar.Fallback>
                                                {suggestion.user_name
                                                    .split(' ')
                                                    .map(word => word[0])
                                                    .join('')
                                                    .toUpperCase()
                                                    .slice(0, 2)}
                                            </Avatar.Fallback>
                                            {suggestion.user_avatar && (
                                                <Avatar.Image src={suggestion.user_avatar} />
                                            )}
                                        </Avatar.Root>
                                        <Box>
                                            <Text fontWeight="bold">{suggestion.user_name}</Text>
                                            <Text fontSize="xs" color="gray.500">
                                                {formatDistanceToNow(new Date(suggestion.created_at), { addSuffix: true })}
                                            </Text>
                                        </Box>
                                    </HStack>
                                    <HStack gap={2}>
                                        <Badge
                                            colorScheme={statusColors[suggestion.status as keyof typeof statusColors]}
                                            display="flex"
                                            alignItems="center"
                                            gap={1}
                                        >
                                            {getStatusIcon(suggestion.status)}
                                            {suggestion.status.replace('_', ' ')}
                                        </Badge>

                                        {/* Admin Menu */}
                                        {isAdmin && (
                                            <Menu.Root>
                                                <Menu.Trigger asChild>
                                                    <Button variant="ghost" size="sm">
                                                        <MoreVertical size={16} />
                                                    </Button>
                                                </Menu.Trigger>
                                                <Menu.Content>
                                                    <Menu.Item
                                                        value="review"
                                                        onClick={() => {
                                                            setSelectedStatus('in_review');
                                                            handleAdminAction(suggestion);
                                                        }}
                                                    >
                                                        <Eye size={16} />
                                                        <Text ml={2}>Mark In Review</Text>
                                                    </Menu.Item>
                                                    <Menu.Item
                                                        value="approve"
                                                        onClick={() => {
                                                            setSelectedStatus('approved');
                                                            handleAdminAction(suggestion);
                                                        }}
                                                    >
                                                        <Check size={16} />
                                                        <Text ml={2}>Approve</Text>
                                                    </Menu.Item>
                                                    <Menu.Item
                                                        value="reject"
                                                        onClick={() => {
                                                            setSelectedStatus('rejected');
                                                            handleAdminAction(suggestion);
                                                        }}
                                                    >
                                                        <X size={16} />
                                                        <Text ml={2}>Reject</Text>
                                                    </Menu.Item>
                                                    <Menu.Item
                                                        value="implement"
                                                        onClick={() => {
                                                            setSelectedStatus('implemented');
                                                            handleAdminAction(suggestion);
                                                        }}
                                                    >
                                                        <CheckCircle size={16} />
                                                        <Text ml={2}>Mark Implemented</Text>
                                                    </Menu.Item>
                                                </Menu.Content>
                                            </Menu.Root>
                                        )}
                                    </HStack>
                                </HStack>
                            </CardHeader>

                            <CardBody>
                                <VStack align="stretch" gap={2}>
                                    <Text fontSize="lg" fontWeight="bold">{suggestion.title}</Text>
                                    <Text color="gray.600">{suggestion.description}</Text>

                                    {suggestion.category && (
                                        <Badge colorScheme="blue" variant="outline" alignSelf="flex-start">
                                            {suggestion.category}
                                        </Badge>
                                    )}

                                    {/* Show admin notes to everyone */}
                                    {suggestion.admin_notes && (
                                        <Box mt={2} p={2} bg="gray.50" borderRadius="md">
                                            <Text fontSize="sm" fontWeight="bold" color="gray.600">
                                                {isAdmin ? 'Your Note:' : 'Admin Note:'}
                                            </Text>
                                            <Text fontSize="sm">{suggestion.admin_notes}</Text>
                                        </Box>
                                    )}
                                </VStack>
                            </CardBody>

                            <CardFooter>
                                <HStack gap={4}>
                                    {/* Upvote button - hide for admins? or keep visible */}
                                    {!isAdmin && (
                                        <Button
                                            size="sm"
                                            variant={suggestion.user_voted ? "solid" : "ghost"}
                                            colorScheme="blue"
                                            onClick={() => handleUpvote(suggestion.id)}
                                            loading={upvoteMutation.isPending}
                                            disabled={suggestion.user_voted}
                                        >
                                            <HStack gap={1}>
                                                <ThumbsUp size={16} />
                                                <Text>{suggestion.upvotes}</Text>
                                            </HStack>
                                        </Button>
                                    )}

                                    {/* Show upvote count to admins without button */}
                                    {isAdmin && (
                                        <HStack gap={1} color="gray.500">
                                            <ThumbsUp size={16} />
                                            <Text>{suggestion.upvotes} upvotes</Text>
                                        </HStack>
                                    )}
                                </HStack>
                            </CardFooter>
                        </Card.Root>
                    ))
                )}
            </VStack>

            {/* Admin Status Update Dialog */}
            {/* Admin Status Update Dialog */}
            {isAdminDialogOpen && (
                <Box
                    position="fixed"
                    top="0"
                    left="0"
                    right="0"
                    bottom="0"
                    bg="blackAlpha.600"
                    zIndex="modal"
                    display="flex"
                    alignItems="center"
                    justifyContent="center"
                    onClick={() => setIsAdminDialogOpen(false)}
                >
                    <Box
                        bg="white"
                        borderRadius="lg"
                        maxW="500px"
                        w="90%"
                        maxH="90vh"
                        overflowY="auto"
                        p={6}
                        onClick={(e) => e.stopPropagation()}
                        boxShadow="xl"
                    >
                        <VStack gap={4} align="stretch">
                            <HStack justify="space-between">
                                <Heading size="md">Update Suggestion Status</Heading>
                                <Button variant="ghost" size="sm" onClick={() => setIsAdminDialogOpen(false)}>
                                    ✕
                                </Button>
                            </HStack>

                            {selectedSuggestion && (
                                <>
                                    <Box p={3} bg="gray.50" borderRadius="md">
                                        <Text fontWeight="bold" fontSize="sm" color="gray.600">Suggestion:</Text>
                                        <Text fontWeight="medium">{selectedSuggestion.title}</Text>
                                    </Box>

                                    <Box>
                                        <Text fontWeight="bold" mb={2}>Status</Text>
                                        <select
                                            className="chakra-select"
                                            value={selectedStatus}
                                            onChange={(e) => setSelectedStatus(e.target.value as UpdateStatusRequest['status'])}
                                            style={{
                                                width: '100%',
                                                padding: '10px',
                                                borderRadius: '8px',
                                                border: '1px solid',
                                                borderColor: '#E2E8F0',
                                                fontSize: '16px',
                                            }}
                                        >
                                            <option value="pending">🕒 Pending</option>
                                            <option value="in_review">👀 In Review</option>
                                            <option value="approved">✅ Approved</option>
                                            <option value="rejected">❌ Rejected</option>
                                            <option value="implemented">🎉 Implemented</option>
                                        </select>
                                    </Box>

                                    <Box>
                                        <Text fontWeight="bold" mb={2}>Admin Notes</Text>
                                        <Textarea
                                            placeholder="Add notes about this decision..."
                                            value={adminNotes}
                                            onChange={(e) => setAdminNotes(e.target.value)}
                                            rows={4}
                                        />
                                    </Box>

                                    <HStack gap={3} justify="flex-end" mt={4}>
                                        <Button variant="ghost" onClick={() => setIsAdminDialogOpen(false)}>
                                            Cancel
                                        </Button>
                                        <Button colorScheme="blue" onClick={handleUpdateStatus}>
                                            Update Status
                                        </Button>
                                    </HStack>
                                </>
                            )}
                        </VStack>
                    </Box>
                </Box>)}
        </Container>
    );
}
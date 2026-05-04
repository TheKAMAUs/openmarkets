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
    Select,
} from "@chakra-ui/react";
import { ArrowUp, Plus, Send, ThumbsUp } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { formatDistanceToNow } from "date-fns";
import { toaster } from "@/components/ui/toaster";
import useUserInfo from "@/hooks/useUserInfo";
import { SuggestionAPI, SuggestionResponse } from "@/utils/interactions/dataGetter";
import { CreateSuggestionRequest, Suggestion, SuggestionAPII } from "@/utils/interactions/dataPosters";




interface SuggestionsClientProps {
    initialSuggestions?: SuggestionResponse[];
}

export default function SuggestionsClient({
    initialSuggestions = [],
}: SuggestionsClientProps) {
    const [suggestions, setSuggestions] = useState<SuggestionResponse[]>(initialSuggestions);
    const { data: userInfo, isLoading: userLoading } = useUserInfo();
    const [showCreateForm, setShowCreateForm] = useState(false);

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
                        data = await SuggestionAPI.getSuggestions();
                    } else {
                        const publicData = await SuggestionAPI.getPublicSuggestions();
                        // Just add user_voted: false to each item
                        data = publicData.map(item => ({
                            ...item,
                            user_voted: false  // 👈 SIMPLE FIX
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
    }, [initialSuggestions.length, userInfo]);

    // Create suggestion mutation
    const createMutation = useMutation<Suggestion, Error, CreateSuggestionRequest>({
        mutationFn: (data) => SuggestionAPII.createSuggestion(data),
    });

    // Upvote mutation
    const upvoteMutation = useMutation<void, Error, string>({
        mutationFn: (suggestionId) => SuggestionAPII.upvoteSuggestion(suggestionId),
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
                    // Transform Suggestion to SuggestionResponse
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
                <Heading size="lg" mb={2}>💡 Market Suggestions</Heading>
                <Text color="gray.600">
                    Suggest new markets for the platform. Upvote ideas you like!
                </Text>
            </Box>

            {/* Create Suggestion Button */}
            {!showCreateForm && (
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

            {/* Create Suggestion Form */}
            {showCreateForm && (
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
                        <Text color="gray.500">No suggestions yet. Be the first to suggest a market!</Text>
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
                                    <Badge
                                        colorScheme={
                                            suggestion.status === 'approved' ? 'green' :
                                                suggestion.status === 'rejected' ? 'red' :
                                                    suggestion.status === 'implemented' ? 'purple' :
                                                        suggestion.status === 'in_review' ? 'yellow' : 'gray'
                                        }
                                    >
                                        {suggestion.status}
                                    </Badge>
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

                                    {suggestion.admin_notes && (
                                        <Box mt={2} p={2} bg="gray.50" borderRadius="md">
                                            <Text fontSize="sm" fontWeight="bold" color="gray.600">Admin Note:</Text>
                                            <Text fontSize="sm">{suggestion.admin_notes}</Text>
                                        </Box>
                                    )}
                                </VStack>
                            </CardBody>

                            <CardFooter>
                                <HStack gap={4}>
                                    <Button
                                        size="sm"
                                        variant={suggestion.user_voted ? "solid" : "ghost"}
                                        colorScheme="blue"
                                        onClick={() => handleUpvote(suggestion.id)}
                                        loading={upvoteMutation.isPending}
                                    >
                                        <HStack gap={1}>
                                            <ThumbsUp size={16} />
                                            <Text>{suggestion.upvotes}</Text>
                                        </HStack>
                                    </Button>
                                </HStack>
                            </CardFooter>
                        </Card.Root>
                    ))
                )}
            </VStack>
        </Container>
    );
}
"use client";

import { useEffect, useState } from "react";
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
    Separator,
    Badge,
    Center,
} from "@chakra-ui/react";
import { ArrowUp, MessageCircle, Send } from "lucide-react";

import { toaster } from "@/components/ui/toaster";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { formatDistanceToNow } from "date-fns";
import { DiscussionAPI, DiscussionResponse } from "@/utils/interactions/dataGetter";
import { CreateDiscussionRequest, Discussion, DiscussionAPII, } from "@/utils/interactions/dataPosters";
import useUserInfo from "@/hooks/useUserInfo";
import { Market, MarketStatus } from "@/generated/grpc_service_types/markets";


interface DiscussionsClientProps {
    initialDiscussions?: DiscussionResponse[];
    marketId: string;
    // market: Market;
}

export default function DiscussionsClient({
    initialDiscussions = [],// Default to empty array
    marketId,
    // market

}: DiscussionsClientProps) {
    const [discussions, setDiscussions] = useState(initialDiscussions);
    const { data: userInfo, isLoading: userLoading } = useUserInfo();
    const [newComment, setNewComment] = useState("");
    const [replyTo, setReplyTo] = useState<string | null>(null);
    const [replyContent, setReplyContent] = useState("");

    const queryClient = useQueryClient();

    // Create discussion mutation with proper typing
    const createMutation = useMutation<Discussion, Error, CreateDiscussionRequest>({
        mutationFn: (data) => DiscussionAPII.createDiscussion(data),
    });
    // Upvote mutation
    const upvoteMutation = useMutation<void, Error, string>({
        mutationFn: (discussionId) => DiscussionAPII.upvoteDiscussion(discussionId),
    });


    // Fetch discussions on mount if not provided
    useEffect(() => {
        if (initialDiscussions.length === 0) {
            const fetchDiscussions = async () => {
                try {
                    const data = await DiscussionAPI.getMarketDiscussions(marketId);
                    setDiscussions(data);
                } catch (error) {
                    console.error("Failed to fetch discussions:", error);
                    toaster.error({
                        title: "Failed to load discussions",
                    });
                }
            };
            fetchDiscussions();
        }
    }, [marketId, initialDiscussions.length]);




    const handleCreateDiscussion = () => {
        if (!userInfo) {
            toaster.info({
                title: "Please login to comment",
                description: "You need to be logged in to join the discussion.",
            });
            return;
        }

        if (!newComment.trim()) {
            toaster.error({
                title: "Empty comment",
                description: "Please write something before posting.",
            });
            return;
        }

        toaster.promise(
            createMutation.mutateAsync({
                market_id: marketId,
                content: newComment,
                parent_id: null,
            }),
            {
                loading: { title: "Posting comment..." },
                success: (newDiscussion) => {
                    // ✅ Transform Discussion to DiscussionResponse
                    const newDiscussionResponse: DiscussionResponse = {
                        id: newDiscussion.id,
                        market_id: newDiscussion.market_id,
                        user: {
                            id: null, // Assuming userInfo has id from GetUserResponse
                            name: userInfo.name,
                            avatar: userInfo.avatar || null,
                        },
                        content: newDiscussion.content,
                        upvotes: newDiscussion.upvotes,
                        reply_count: 0,
                        created_at: newDiscussion.created_at,
                    };

                    setDiscussions([newDiscussionResponse, ...discussions]);
                    setNewComment("");
                    queryClient.invalidateQueries({ queryKey: ["discussions", marketId] });
                    return { title: "Comment posted successfully!" };
                },
                error: (e) => ({
                    title: "Failed to post comment",
                    description: e instanceof Error ? e.message : "An error occurred",
                }),
            }
        );
    };
    const handleUpvote = (discussionId: string) => {
        if (!userInfo) {
            toaster.info({
                title: "Please login to upvote",
                description: "You need to be logged in to upvote discussions.",
            });
            return;
        }

        toaster.promise(
            upvoteMutation.mutateAsync(discussionId),
            {
                loading: { title: "Upvoting..." },
                success: () => {
                    setDiscussions(discussions.map(d =>
                        d.id === discussionId
                            ? { ...d, upvotes: d.upvotes + 1 }
                            : d
                    ));
                    queryClient.invalidateQueries({ queryKey: ["discussions", marketId] });
                    return { title: "Upvoted successfully!" };
                },
                error: (e) => ({
                    title: "Failed to upvote",
                    description: e instanceof Error ? e.message : "An error occurred",
                }),
            }
        );
    };

    const handleReply = (parentId: string) => {
        if (!userInfo) {
            toaster.info({
                title: "Please login to reply",
                description: "You need to be logged in to reply.",
            });
            return;
        }

        if (!replyContent.trim()) {
            toaster.error({
                title: "Empty reply",
                description: "Please write something before replying.",
            });
            return;
        }

        toaster.promise(
            createMutation.mutateAsync({
                market_id: marketId,
                content: replyContent,  // ✅ FIXED: was using newComment, now using replyContent
                parent_id: parentId,
            }),
            {
                loading: { title: "Posting reply..." },
                success: (newReply) => {
                    // Refresh discussions to show the new reply
                    DiscussionAPI.getMarketDiscussions(marketId).then(updated => {
                        setDiscussions(updated);
                    });
                    setReplyTo(null);
                    setReplyContent("");
                    queryClient.invalidateQueries({ queryKey: ["discussions", marketId] });
                    return { title: "Reply posted successfully!" };
                },
                error: (e) => ({
                    title: "Failed to post reply",
                    description: e instanceof Error ? e.message : "An error occurred",
                }),
            }
        );
    };

    return (
        <Container maxW="4xl" py={8}>
            {/* Header */}
            <Box mb={8}>
                <Heading size="lg" mb={2}>💬 Market Discussions</Heading>
                <Text color="gray.600">
                    Join the conversation about this market
                </Text>
            </Box>

            {/* New Comment Form */}
            <Box mb={8} p={4} bg="gray.50" borderRadius="lg">
                <Textarea
                    placeholder={userInfo ? "Share your thoughts..." : "Please login to comment"}
                    value={newComment}
                    onChange={(e) => setNewComment(e.target.value)}
                    mb={3}
                    rows={3}
                    disabled={!userInfo}
                />
                <Button
                    colorScheme="blue"
                    onClick={handleCreateDiscussion}
                    loading={createMutation.isPending}
                    disabled={

                        // market.status === MarketStatus.SETTLED
                        // ||
                        !userInfo || !newComment.trim()}
                >
                    <HStack gap={2}>
                        <Text>Post Comment</Text>
                        <Send size={16} />
                    </HStack>
                </Button>
            </Box>

            <Separator mb={6} />  {/* ✅ Changed from Divider to Separator */}

            {/* Discussions List */}
            <VStack gap={4} align="stretch">
                {discussions.length === 0 ? (
                    <Center py={10}>
                        <Text color="gray.500">Start the Discussion!</Text>
                    </Center>
                ) : (
                    discussions.map((discussion) => (
                        <Box key={discussion.id} p={4} borderWidth="1px" borderRadius="lg">
                            {/* Discussion Header */}
                            <HStack justify="space-between" mb={3}>
                                <HStack gap={2}>
                                    <Avatar.Root size="sm">
                                        <Avatar.Fallback>
                                            {discussion.user.name.includes(' ')
                                                ? discussion.user.name
                                                    .split(' ')
                                                    .map(word => word[0])
                                                    .join('')
                                                    .toUpperCase()
                                                    .slice(0, 2)
                                                : discussion.user.name.slice(0, 2).toUpperCase()
                                            }
                                        </Avatar.Fallback>
                                        {discussion.user.avatar && <Avatar.Image src={discussion.user.avatar} />}

                                    </Avatar.Root>
                                    <Box>
                                        <Text fontWeight="bold">{discussion.user.name}</Text>
                                        <Text fontSize="xs" color="gray.500">
                                            {formatDistanceToNow(new Date(discussion.created_at), { addSuffix: true })}
                                        </Text>
                                    </Box>
                                </HStack>
                                <Badge colorScheme="blue" variant="outline">
                                    {discussion.reply_count} {discussion.reply_count === 1 ? 'reply' : 'replies'}
                                </Badge>
                            </HStack>

                            {/* Discussion Content */}
                            <Text mb={4} whiteSpace="pre-wrap">
                                {discussion.content}
                            </Text>

                            {/* Actions */}
                            <HStack gap={4}>
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    onClick={() => handleUpvote(discussion.id)}
                                    loading={upvoteMutation.isPending}
                                    asChild
                                >
                                    <span>
                                        <ArrowUp size={16} style={{ marginRight: '4px' }} />
                                        {discussion.upvotes}
                                    </span>
                                </Button>
                                {/* <Button
                                    size="sm"
                                    variant="ghost"
                                    onClick={() => setReplyTo(replyTo === discussion.id ? null : discussion.id)}
                                    asChild
                                >
                                    <span>
                                        <MessageCircle size={16} style={{ marginRight: '8px' }} />
                                        Reply
                                    </span>
                                </Button> */}
                            </HStack>

                            {/* Reply Form */}
                            {replyTo === discussion.id && (
                                <Box mt={4} pl={6} borderLeftWidth="2px" borderColor="gray.200">
                                    <Textarea
                                        placeholder={userInfo ? "Write your reply..." : "Please login to reply"}
                                        value={replyContent}
                                        onChange={(e) => setReplyContent(e.target.value)}
                                        size="sm"
                                        mb={2}
                                        rows={2}
                                        disabled={!userInfo}
                                    />
                                    <HStack gap={2}>
                                        <Button
                                            size="sm"
                                            colorScheme="blue"
                                            onClick={() => handleReply(discussion.id)}
                                            loading={createMutation.isPending}
                                            disabled={!userInfo || !replyContent.trim()}
                                        >
                                            Post Reply
                                        </Button>
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            onClick={() => {
                                                setReplyTo(null);
                                                setReplyContent("");
                                            }}
                                        >
                                            Cancel
                                        </Button>
                                    </HStack>
                                </Box>
                            )}
                        </Box>
                    ))
                )}
            </VStack>
        </Container>
    );
}
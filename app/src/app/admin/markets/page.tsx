"use client";

import React from "react";
import { useQueries } from "@tanstack/react-query";
import {
    Container,
    Heading,
    Text,
    Grid,
    GridItem,
    VStack,
    Box,
    Spinner,
    Center,
    HStack,
    Button,
} from "@chakra-ui/react";
import { useMemo, useState } from "react";

import { MarketGetters } from "@/utils/interactions/dataGetter";
import { MarketStatus } from "@/generated/grpc_service_types/markets";
import TrendingMarketCardAdmin from "@/components/TrendingMarketCardAdmin";

export default function AdminMarketsPage() {
    const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

    const [{ data: openMarkets = [], isLoading }, { data: closedMarkets = [] }] = useQueries({
        queries: [
            {
                queryKey: ["marketData", 1, 10, "open"],
                queryFn: () => MarketGetters.getMarketData(1, 10, MarketStatus.OPEN),
            },
            {
                queryKey: ["marketData", 1, 10, "closed"],
                queryFn: () => MarketGetters.getMarketData(1, 10, MarketStatus.SETTLED),
            },
        ],
    });

    // Filter out child markets (markets with parent_id)
    const parentOpenMarkets = useMemo(() => {
        return openMarkets.filter(market => !market.parentId || market.parentId === "");
    }, [openMarkets]);

    const parentClosedMarkets = useMemo(() => {
        return closedMarkets.filter(market => !market.parentId || market.parentId === "");
    }, [closedMarkets]);

    // Combine all parent markets to extract unique categories
    const allMarkets = useMemo(() => [...parentOpenMarkets, ...parentClosedMarkets], [parentOpenMarkets, parentClosedMarkets]);

    // Extract unique categories (filtering out null/undefined/empty strings)
    const categories = useMemo(() => {
        const categorySet = new Set<string>();
        allMarkets.forEach(market => {
            if (market.category && market.category.trim() !== "") {
                categorySet.add(market.category);
            }
        });
        return Array.from(categorySet).sort();
    }, [allMarkets]);

    // Filter markets based on selected category
    const filteredOpenMarkets = useMemo(() => {
        if (!selectedCategory) return parentOpenMarkets;
        return parentOpenMarkets.filter(market => market.category === selectedCategory);
    }, [parentOpenMarkets, selectedCategory]);

    const filteredClosedMarkets = useMemo(() => {
        if (!selectedCategory) return parentClosedMarkets;
        return parentClosedMarkets.filter(market => market.category === selectedCategory);
    }, [parentClosedMarkets, selectedCategory]);

    return (
        <Container maxW="7xl" py={10}>
            <VStack align="start" gap={8}>
                {/* Header */}
                <Box>
                    <Heading size="lg">Admin Markets</Heading>
                    <Text color="gray.500">
                        Manage and monitor prediction markets
                    </Text>
                </Box>

                {/* Category Filter Buttons - Horizontal Scroll */}
                {categories.length > 0 && (
                    <Box w="full">
                        <Text fontSize="lg" fontWeight="medium" mb={3}>
                            Filter by Category
                        </Text>
                        <HStack
                            gap={2}
                            overflowX="auto"
                            py={2}
                            pb={4}
                            css={{
                                "&::-webkit-scrollbar": { height: "6px" },
                                "&::-webkit-scrollbar-thumb": { bg: "gray.300", borderRadius: "4px" },
                            }}
                        >
                            <Button
                                size="sm"
                                colorScheme={selectedCategory === null ? "blue" : "gray"}
                                onClick={() => setSelectedCategory(null)}
                                variant={selectedCategory === null ? "solid" : "outline"}
                                borderRadius="full"
                                minW="fit-content"
                            >
                                All
                            </Button>
                            {categories.map((category) => (
                                <Button
                                    key={category}
                                    size="sm"
                                    colorScheme={selectedCategory === category ? "blue" : "gray"}
                                    onClick={() => setSelectedCategory(category)}
                                    variant={selectedCategory === category ? "solid" : "outline"}
                                    borderRadius="full"
                                    minW="fit-content"
                                >
                                    {category}
                                </Button>
                            ))}
                        </HStack>
                    </Box>
                )}

                {/* Loading State */}
                {isLoading && (
                    <Center w="full" py={20}>
                        <Spinner size="lg" />
                    </Center>
                )}

                {/* Open Markets Section */}
                {!isLoading && (
                    <Box w="full">
                        <Text fontSize="2xl" fontWeight="bold" mb={4}>
                            Open Markets
                            {selectedCategory && (
                                <Text as="span" fontSize="md" fontWeight="normal" color="gray.500" ml={2}>
                                    • Filtered by: {selectedCategory}
                                </Text>
                            )}
                        </Text>
                        {filteredOpenMarkets.length > 0 ? (
                            <Grid
                                templateColumns="repeat(auto-fill, minmax(320px, 1fr))"
                                gap={6}
                                w="full"
                            >
                                {filteredOpenMarkets.map((market) => (
                                    <GridItem key={market.id}>
                                        <TrendingMarketCardAdmin market={market} />
                                    </GridItem>
                                ))}
                            </Grid>
                        ) : (
                            <Text color="gray.500" py={4}>
                                No open markets {selectedCategory ? `in ${selectedCategory}` : ""}
                            </Text>
                        )}
                    </Box>
                )}

                {/* Closed Markets Section */}
                {!isLoading && parentClosedMarkets.length > 0 && (
                    <Box w="full">
                        <Text fontSize="2xl" fontWeight="bold" mb={4}>
                            Recently Closed
                            {selectedCategory && (
                                <Text as="span" fontSize="md" fontWeight="normal" color="gray.500" ml={2}>
                                    • Filtered by: {selectedCategory}
                                </Text>
                            )}
                        </Text>
                        {filteredClosedMarkets.length > 0 ? (
                            <Grid
                                templateColumns="repeat(auto-fill, minmax(320px, 1fr))"
                                gap={6}
                                w="full"
                            >
                                {filteredClosedMarkets.map((market) => (
                                    <GridItem key={market.id}>
                                        <TrendingMarketCardAdmin market={market} />
                                    </GridItem>
                                ))}
                            </Grid>
                        ) : (
                            <Text color="gray.500" py={4}>
                                No closed markets {selectedCategory ? `in ${selectedCategory}` : ""}
                            </Text>
                        )}
                    </Box>
                )}

                {/* Empty State */}
                {!isLoading && filteredOpenMarkets.length === 0 && filteredClosedMarkets.length === 0 && (
                    <Center w="full" py={20}>
                        <Text color="gray.500">No markets available</Text>
                    </Center>
                )}
            </VStack>
        </Container>
    );
}
"use client";

import { Container, HStack, VStack, Text, Box, Button, Wrap, WrapItem, Link, Separator } from "@chakra-ui/react";
import { useQueries } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import NextLink from "next/link";

import { MarketGetters } from "@/utils/interactions/dataGetter";
import TrendingMarketCard from "@/components/TrendingMarketCard";
import { Market, MarketStatus } from "@/generated/grpc_service_types/markets";
import { useMarketStore } from "@/hooks/store/marketStore";
import { useRouter } from "next/navigation";

export default function Home() {
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

  const [{ data: openMarkets = [] }, { data: closedMarkets = [] }] = useQueries({
    queries: [
      {
        queryKey: ["marketData", 1, 30, "open"],
        queryFn: () => MarketGetters.getMarketData(1, 30, MarketStatus.OPEN),
      },
      {
        queryKey: ["recentlyClosedMarkets", 1, 10, "closed"],
        queryFn: () => MarketGetters.getMarketData(1, 10, MarketStatus.SETTLED),
      },
    ],
  });
  const router = useRouter();
  const setMarket = useMarketStore((s) => s.setMarket);
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


  const cleanedOpenMarkets = filteredOpenMarkets.filter(
    (market) => market.name !== "BTCUSDT"
  );

  const btcRecentClosedMarkets = useMemo(() => {
    const THIRTY_MIN = 30 * 60 * 1000;
    const now = Date.now();

    return filteredClosedMarkets
      .filter((market) => {
        const isBTC = market.name === "BTCUSDT";
        if (!isBTC) return false;

        const updatedAt =
          new Date(market.updatedAt).getTime() + 3 * 60 * 60 * 1000;

        if (isNaN(updatedAt)) return false;

        return now - updatedAt <= THIRTY_MIN;
      })
      .sort(
        (a, b) =>
          (new Date(a.updatedAt).getTime() + 3 * 60 * 60 * 1000) -
          (new Date(b.updatedAt).getTime() + 3 * 60 * 60 * 1000)
      )
      .slice(0, 6);
  }, [filteredClosedMarkets]);



  return (
    <Container maxW="7xl" py={10}>
      <VStack align="start" gap={8}>

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

        {/* Trending Markets Section */}
        <Box w="full">
          <Text fontSize="2xl" fontWeight="bold" mb={4}>
            Trending Markets
            {selectedCategory && (
              <Text as="span" fontSize="md" fontWeight="normal" color="gray.500" ml={2}>
                • Filtered by: {selectedCategory}
              </Text>
            )}
          </Text>
          {cleanedOpenMarkets.length > 0 ? (
            <HStack
              gap={4}
              overflowX="auto"
              py={2}
              css={{
                "&::-webkit-scrollbar": { height: "6px" },
                "&::-webkit-scrollbar-thumb": { bg: "gray.300", borderRadius: "4px" },
              }}
            >
              {cleanedOpenMarkets.map((market) => (
                <TrendingMarketCard key={market.id} market={market} />
              ))}
            </HStack>
          ) : (
            <Text color="gray.500" py={4}>
              No open markets {selectedCategory ? `in ${selectedCategory}` : ""}
            </Text>
          )}
        </Box>

        <Box w="full" display="flex" justifyContent="center">
          <Box w="100%" maxW="md">

            <Box
              as="button"
              onClick={() => {
                setMarket(null);
                router.push("/BTC");
              }}
              borderWidth="1px"
              borderRadius="xl"
              p={4}
              bg="white"
              _dark={{ bg: "gray.800" }}
              w="100%"
              textAlign="left"
              _hover={{ shadow: "md", transform: "scale(1.01)" }}
              transition="0.15s ease"
            >
              <HStack justify="space-between">
                <HStack>
                  <Box w="10px" h="10px" bg="green.400" borderRadius="full" />
                  <Text fontWeight="bold">BTC / USDT</Text>
                </HStack>
                <Text fontSize="sm" color="gray.500">Live</Text>
              </HStack>

              <HStack justify="space-between" mt={3}>
                <Text fontWeight="semibold">$64,200</Text>
                <Text fontSize="xs" color="green.400">+2.4%</Text>
              </HStack>
            </Box>

            {/* Buttons BELOW card */}
            <HStack mt={3} gap={2} wrap="wrap" justify="center">
              {btcRecentClosedMarkets.map((market: Market) => {
                const time = new Date(market.updatedAt).toLocaleTimeString([], {
                  hour: "2-digit",
                  minute: "2-digit",
                });

                return (
                  <Button
                    key={market.id}
                    size="xs"
                    variant="outline"
                    borderRadius="full"
                    onClick={() => {
                      setMarket(market);
                      router.push("/BTC");
                    }}
                  >
                    {time}
                  </Button>
                );
              })}
            </HStack>

          </Box>
        </Box>



        {/* Recently Closed Section */}
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
            <HStack
              gap={4}
              overflowX="auto"
              py={2}
              css={{
                "&::-webkit-scrollbar": { height: "6px" },
                "&::-webkit-scrollbar-thumb": { bg: "gray.300", borderRadius: "4px" },
              }}
            >
              {filteredClosedMarkets.map((market) => (
                <TrendingMarketCard key={market.id} market={market} />
              ))}
            </HStack>
          ) : (
            <Text color="gray.500" py={4}>
              No closed markets {selectedCategory ? `in ${selectedCategory}` : ""}
            </Text>
          )}
        </Box>

      </VStack>

      {/* <Link
        href="/auth"
        style={{
          background: 'blue',
          color: 'white',
          padding: '10px 20px',
          borderRadius: '5px',
          textDecoration: 'none',
          display: 'inline-block'
        }}
      >
        Login
      </Link> */}

      {/* <Separator mb={6} />  ✅ Changed from Divider to Separator */}
      {/* <Link
        href="/suggestions"
        style={{
          background: 'blue',
          color: 'white',
          padding: '10px 20px',
          borderRadius: '5px',
          textDecoration: 'none',
          display: 'inline-block'
        }}
      >
        Suggestions
      </Link>


      <Link
        href="/admin/suggestions"
        style={{
          background: 'blue',
          color: 'white',
          padding: '10px 20px',
          borderRadius: '5px',
          textDecoration: 'none',
          display: 'inline-block'
        }}
      >
        Suggestions
      </Link>
      <Link
        href="/leaderboard"
        style={{
          background: 'blue',
          color: 'white',
          padding: '10px 20px',
          borderRadius: '5px',
          textDecoration: 'none',
          display: 'inline-block'
        }}
      >
        leaderboard
      </Link>

      <Link
        href="/liquidity"
        style={{
          background: 'blue',
          color: 'white',
          padding: '10px 20px',
          borderRadius: '5px',
          textDecoration: 'none',
          display: 'inline-block'
        }}
      >
        liquidity
      </Link> */}
      {/* <Link
        href="/BTC"
        style={{
          background: 'blue',
          color: 'white',
          padding: '10px 20px',
          borderRadius: '5px',
          textDecoration: 'none',
          display: 'inline-block'
        }}
      >
        BTC
      </Link> */}

    </Container>
  );
}
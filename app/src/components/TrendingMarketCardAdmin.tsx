"use client";

import {
    Box,
    Card,
    CardBody,
    CardFooter,
    Button,
    Avatar,
    Heading,
    Text,
    HStack,
    VStack,
    Badge,
    Flex,
    Icon,
    Progress,
    Separator,
} from "@chakra-ui/react";
import {
    TrendingUp,
    Clock,
    Activity,
    Calendar,
    BarChart3,
} from "lucide-react";
import Link from "next/link";
import { useColorModeValue } from "./ui/color-mode";
import { Market, MarketStatus } from "@/generated/grpc_service_types/markets";
import { AdminMarketButtons } from "./AdminMarketButtons";
import { getImageColors } from "./ui/color-mode";
import { useEffect, useMemo, useState } from "react";
import { useQueries, useQuery } from "@tanstack/react-query";
import { MarketGetters } from "@/utils/interactions/dataGetter";

interface Props {
    market: Market;
}

function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
    });
}

function getStatusColor(status: MarketStatus) {
    switch (status) {
        case MarketStatus.OPEN:
            return "green";
        case MarketStatus.CLOSED:
            return "orange";
        case MarketStatus.SETTLED:
            return "blue";
        default:
            return "gray";
    }
}

function getTimeRemaining(expiryDate: string) {
    const now = new Date();
    const expiry = new Date(expiryDate);
    const diffTime = expiry.getTime() - now.getTime();
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

    if (diffDays < 0) return "Expired";
    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "1 day";
    return `${diffDays} days`;
}

function getClosingSoonStatus(expiryDate: string) {
    const now = new Date();
    const expiry = new Date(expiryDate);
    const diffTime = expiry.getTime() - now.getTime();
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    return diffDays <= 30 && diffDays >= 0;
}

// Helper function to format volume
function formatVolume(usd: number): string {
    if (usd >= 1_000_000) {
        return `$${(usd / 1_000_000).toFixed(1)}M`;
    } else if (usd >= 1_000) {
        return `$${(usd / 1_000).toFixed(1)}K`;
    } else {
        return `$${usd.toFixed(0)}`;
    }
}

// Calculate total volume from VolumeInfo
function calculateTotalVolume(volumeInfo: any): number {
    if (!volumeInfo) return 0;
    return (
        (volumeInfo.yesBuyUsd || 0) +
        (volumeInfo.yesSellUsd || 0) +
        (volumeInfo.noBuyUsd || 0) +
        (volumeInfo.noSellUsd || 0)
    );
}

const TrendingMarketCardAdmin = ({ market }: Props) => {
    const cardBg = useColorModeValue("white", "gray.800");
    const borderColor = useColorModeValue("gray.200", "gray.600");
    const borderColour = useColorModeValue("white", "gray.800");
    const closingSoonBg = useColorModeValue("orange.500", "orange.400");
    const closingSoonColor = useColorModeValue("white", "white");

    // Fetch full market details to get child markets, volume info, and market prices
    const { data: marketResponse } = useQuery({
        queryKey: ["market", market.id, "full"],
        queryFn: () => MarketGetters.getMarketById(market.id),
        enabled: market.isEvent, // Only fetch if it's an event
    });

    const fullMarket = marketResponse?.market;
    const volumeInfo = marketResponse?.volumeInfo;
    const marketPrice = marketResponse?.marketPrice;

    // Use market price from response instead of the prop
    const yesPrice = marketPrice?.latestYesPrice ?? market.yesPrice;
    const noPrice = marketPrice?.latestNoPrice ?? market.noPrice;

    // Calculate total volume
    const totalVolume = calculateTotalVolume(volumeInfo);
    const formattedVolume = formatVolume(totalVolume);

    const yesPercentage = (yesPrice * 100).toFixed(0);
    const noPercentage = (noPrice * 100).toFixed(0);
    const timeRemaining = getTimeRemaining(market.marketExpiry);
    const isClosingSoon = getClosingSoonStatus(market.marketExpiry);

    // Fetch child markets if this is an event
    const childMarketIds = fullMarket?.childMarkets?.map(m => m.id) || [];
    const childMarketQueries = useQueries({
        queries: childMarketIds.map(id => ({
            queryKey: ["market", id, "child"],
            queryFn: () => MarketGetters.getMarketById(id),
            enabled: market.isEvent && childMarketIds.length > 0,
        })),
    });

    const [firstButtonColors, setFirstButtonColors] = useState<{ primary: string; secondary: string } | null>(null);
    const [secondButtonColors, setSecondButtonColors] = useState<{ primary: string; secondary: string } | null>(null);
    const textColor = useColorModeValue('gray.800', 'white');


    const isMarketInitialized = useMemo(() => {
        // Check if the market has actual prices (not the default 0.5/0.5)
        // Or check if liquidity exists
        const hasRealPrices = yesPrice !== 0.5 || noPrice !== 0.5;

        console.log(`📊 :`, {
            yesPrice: yesPrice.toFixed(6),
            noPrice: noPrice.toFixed(6),

            hasRealPrices
        });
        // Market is initialized if it has real prices OR has liquidity
        // OR if it's not in PENDING status
        return hasRealPrices;
    }, [yesPrice, noPrice, , market.status]);


    // In TrendingMarketCardAdmin, add this right before the child markets section:
    console.log("🔍 Checking if should render child markets:", {
        isEvent: market.isEvent,
        hasChildMarkets: !!fullMarket?.childMarkets,
        childMarketsLength: fullMarket?.childMarkets?.length,
        fullMarketExists: !!fullMarket
    });

    useEffect(() => {
        if (market.logo && market.logo.length >= 2) {
            getImageColors(market.logo[0]).then(colors => {
                setFirstButtonColors(colors);
            }).catch(console.error);
            getImageColors(market.logo[1]).then(colors => {
                setSecondButtonColors(colors);
            }).catch(console.error);
        } else {
            setFirstButtonColors(null);
            setSecondButtonColors(null);
        }
    }, [market.logo]);

    const linkHref = market.isEvent ? `/event/${market.id}?fromAdmin=true` : `/market/${market.id}`;

    return (
        <Card.Root
            width="400px"
            bg={cardBg}
            borderColor={borderColor}
            borderWidth="1px"
            borderRadius="xl"
            cursor="pointer"
            transition="all 0.2s"
            overflow="hidden"
            _hover={{ shadow: "md" }}
        >
            <Link href={linkHref} style={{ textDecoration: "none" }}>
                <CardBody p={5} pb={market.isEvent && fullMarket?.childMarkets?.length ? 2 : 5}>
                    <VStack gap={4} align="stretch">
                        {/* Header: Avatar + Time + Status */}
                        <Flex justify="space-between" align="start">
                            <HStack gap={3}>
                                <Avatar.Root size="md" borderRadius="full">
                                    <Avatar.Image
                                        boxSize="40px"
                                        objectFit="cover"
                                        src={market.logo && market.logo.length > 0 ? market.logo[0] : undefined}
                                        alt={market.name}
                                    />
                                    <Avatar.Fallback>
                                        {market.name.includes(' ')
                                            ? market.name.split(' ').map(word => word[0]).join('').toUpperCase().slice(0, 2)
                                            : market.name.slice(0, 2).toUpperCase()
                                        }
                                    </Avatar.Fallback>
                                </Avatar.Root>

                                <VStack align="start" gap={1} flex={1}>
                                    <HStack gap={1} color="gray.500">
                                        <Icon boxSize={3}><Clock /></Icon>
                                        <Text fontSize="xs">{timeRemaining} left</Text>
                                    </HStack>
                                </VStack>
                            </HStack>

                            <HStack gap={2}>
                                {isClosingSoon && market.status === MarketStatus.OPEN && (
                                    <Badge
                                        bg={closingSoonBg}
                                        color={closingSoonColor}
                                        size="sm"
                                        borderRadius="full"
                                        px={2}
                                        py={1}
                                    >
                                        Closing Soon
                                    </Badge>
                                )}
                                <Badge
                                    colorScheme={getStatusColor(market.status)}
                                    variant="solid"
                                    size="sm"
                                    borderRadius="full"
                                >
                                    {MarketStatus[market.status]}
                                </Badge>
                            </HStack>
                        </Flex>

                        {/* Market Title */}
                        <Box>
                            <Heading
                                size="sm"
                                lineHeight="1.3"
                                lineClamp={2}
                                color={useColorModeValue("gray.800", "white")}
                            >
                                {market.name}
                            </Heading>
                        </Box>

                        {/* Market Odds - Only for non-event markets */}
                        {!market.isEvent && (
                            <VStack gap={2} align="stretch">
                                <HStack gap={2} w="full">
                                    {/* Buy Yes Button */}
                                    <Button
                                        flex={1}
                                        size="sm"
                                        onClick={(e) => { e.preventDefault(); e.stopPropagation(); }}
                                        bg="transparent"
                                        position="relative"
                                        overflow="hidden"
                                        color={textColor}
                                        fontWeight="bold"
                                        border="1px solid"
                                        borderColor="rgba(255, 255, 255, 0.2)"
                                        backdropFilter="blur(12px)"
                                        transition="all 0.3s ease"
                                        _before={{
                                            content: '""',
                                            position: 'absolute',
                                            top: 0,
                                            left: 0,
                                            right: 0,
                                            bottom: 0,
                                            background: firstButtonColors
                                                ? `linear-gradient(135deg, ${firstButtonColors.primary}, ${firstButtonColors.secondary})`
                                                : useColorModeValue('linear-gradient(135deg, #22c55e, #15803d)', 'linear-gradient(135deg, #15803d, #22c55e)'),
                                            opacity: 0.6,
                                            zIndex: 0,
                                            transition: 'opacity 0.3s ease',
                                        }}
                                        _hover={{
                                            transform: 'scale(1.02)',
                                            _before: { opacity: 0.9 }
                                        }}
                                        _active={{ transform: 'scale(0.98)' }}
                                        css={{ '& > *': { position: 'relative', zIndex: 1 } }}
                                    >
                                        <TrendingUp size={16} style={{ marginRight: '8px' }} />
                                        Buy Yes
                                    </Button>

                                    {/* Buy No Button */}
                                    <Button
                                        flex={1}
                                        size="sm"
                                        onClick={(e) => { e.preventDefault(); e.stopPropagation(); }}
                                        bg="transparent"
                                        position="relative"
                                        overflow="hidden"
                                        color={textColor}
                                        fontWeight="bold"
                                        border="1px solid"
                                        borderColor="rgba(255, 255, 255, 0.2)"
                                        backdropFilter="blur(12px)"
                                        transition="all 0.3s ease"
                                        _before={{
                                            content: '""',
                                            position: 'absolute',
                                            top: 0,
                                            left: 0,
                                            right: 0,
                                            bottom: 0,
                                            background: secondButtonColors
                                                ? `linear-gradient(135deg, ${secondButtonColors.primary}, ${secondButtonColors.secondary})`
                                                : useColorModeValue('linear-gradient(135deg, #ef4444, #991b1b)', 'linear-gradient(135deg, #991b1b, #ef4444)'),
                                            opacity: 0.6,
                                            zIndex: 0,
                                            transition: 'opacity 0.3s ease',
                                        }}
                                        _hover={{
                                            transform: 'scale(1.02)',
                                            _before: { opacity: 0.9 }
                                        }}
                                        _active={{ transform: 'scale(0.98)' }}
                                        css={{ '& > *': { position: 'relative', zIndex: 1 } }}
                                    >
                                        <Activity size={16} style={{ marginRight: '8px' }} />
                                        Buy No
                                    </Button>
                                </HStack>
                            </VStack>
                        )}

                        {/* Volume - Only for non-event markets */}
                        {!market.isEvent && (
                            <HStack justify="flex-end">
                                <HStack gap={1} color="gray.500" fontSize="xs">
                                    <Icon boxSize={3}><BarChart3 /></Icon>
                                    <Text>Vol {formattedVolume}</Text>
                                </HStack>
                            </HStack>
                        )}

                        {/* Child Markets Section - Only show if this is an event with children */}
                        {market.isEvent && fullMarket?.childMarkets && fullMarket.childMarkets.length > 0 && (
                            <Box>
                                <VStack gap={0}
                                    maxH="120px"
                                    overflowY="auto"
                                    pr={1}
                                    css={{
                                        "&::-webkit-scrollbar": { width: "4px" },
                                        "&::-webkit-scrollbar-thumb": { bg: "gray.300", borderRadius: "4px" },
                                    }}
                                >
                                    {fullMarket?.childMarkets.map((childMarket, index) => {
                                        const childResponse = childMarketQueries[index]?.data;

                                        const childVolumeInfo = childResponse?.volumeInfo;
                                        const childMarketPrice = childResponse?.marketPrice;

                                        // Use child market's own price data
                                        const childYesPrice = childMarketPrice?.latestYesPrice ?? childMarket.yesPrice;
                                        const childNoPrice = childMarketPrice?.latestNoPrice ?? childMarket.noPrice;

                                        // Calculate child volume
                                        const childTotalVolume = calculateTotalVolume(childVolumeInfo);
                                        const childFormattedVolume = formatVolume(childTotalVolume);

                                        return (
                                            <Box
                                                key={childMarket.id}
                                                w="full"
                                                p={2}
                                                bg={useColorModeValue("white", "gray.800")}
                                                borderRadius="md"
                                                borderWidth="1px"
                                                borderColor={borderColour}
                                                onClick={(e) => {
                                                    e.preventDefault();
                                                    e.stopPropagation();
                                                    window.location.href = `/market/${childMarket.id}`;
                                                }}
                                            >
                                                <HStack justify="space-between" w="full">
                                                    <VStack align="start" gap={0} flex={1}>
                                                        <Text fontSize="xs" fontWeight="medium" lineClamp={1}>
                                                            {childMarket.name}
                                                        </Text>
                                                        <HStack gap={2} mt={1}>
                                                            <HStack gap={1}>
                                                                <Text fontSize="2xs" color="green.500">YES</Text>
                                                                <Text fontSize="xs" color="green.500" fontWeight="bold">
                                                                    {(childYesPrice * 100).toFixed(0)}%
                                                                </Text>
                                                            </HStack>
                                                            <HStack gap={1}>
                                                                <Text fontSize="2xs" color="red.500">NO</Text>
                                                                <Text fontSize="xs" color="red.500" fontWeight="bold">
                                                                    {(childNoPrice * 100).toFixed(0)}%
                                                                </Text>
                                                            </HStack>
                                                        </HStack>
                                                    </VStack>
                                                    <HStack gap={1} color="gray.500" fontSize="2xs">
                                                        <Icon boxSize={2}><BarChart3 /></Icon>
                                                        <Text>{childFormattedVolume}</Text>
                                                    </HStack>
                                                </HStack>
                                            </Box>
                                        );
                                    })}
                                </VStack>
                            </Box>
                        )}
                    </VStack>
                </CardBody>
            </Link>


            {market.status === MarketStatus.OPEN && (
                <CardFooter pt={0} pb={4} px={5}>
                    <Box w="full">
                        <AdminMarketButtons
                            market={market}
                            isInitialized={isMarketInitialized}
                        />
                    </Box>
                </CardFooter>
            )}
        </Card.Root>
    );
};

export default TrendingMarketCardAdmin;
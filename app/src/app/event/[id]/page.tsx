// app/event/[id]/page.tsx
"use client";

import { useParams, useRouter, useSearchParams } from "next/navigation";
import {
    Container,
    VStack,
    HStack,
    Heading,
    Text,
    Box,
    Button,
    Avatar,
    Badge,
    Card,
    CardBody,
    Icon,
    SimpleGrid,
    Separator,
    Spinner,
    Center,

    Flex,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import {
    ArrowLeft,
    Calendar,
    Clock,
    BarChart3,
    TrendingUp,
    Activity,
    Users,
    ChevronRight,
} from "lucide-react";
import Link from "next/link";
import { MarketGetters } from "@/utils/interactions/dataGetter";
import { MarketStatus } from "@/generated/grpc_service_types/markets";
import { getImageColors, useColorModeValue, useColorModeValue as useColorModeValueTheme } from "@/components/ui/color-mode";
import { useState, useEffect, useMemo } from "react";
import { AdminMarketButtons } from "@/components/AdminMarketButtons";

// Helper functions
function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString("en-US", {
        month: "long",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
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

function formatVolume(usd: number): string {
    if (usd >= 1_000_000) {
        return `$${(usd / 1_000_000).toFixed(1)}M`;
    } else if (usd >= 1_000) {
        return `$${(usd / 1_000).toFixed(1)}K`;
    } else {
        return `$${usd.toFixed(0)}`;
    }
}

function calculateTotalVolume(volumeInfo: any): number {
    if (!volumeInfo) return 0;
    return (
        (volumeInfo.yesBuyUsd || 0) +
        (volumeInfo.yesSellUsd || 0) +
        (volumeInfo.noBuyUsd || 0) +
        (volumeInfo.noSellUsd || 0)
    );
}


// Child Market Card Component - Compact version
const ChildMarketCard = ({ childMarket, isAdminView = false }: { childMarket: any, isAdminView?: boolean; }) => {
    const router = useRouter();
    const cardBg = useColorModeValue("gray.50", "gray.700");
    const borderColor = useColorModeValue("gray.200", "gray.600");
    const textColor = useColorModeValue("gray.800", "white");

    // Fetch full market details to get latest prices and volume
    const { data: marketResponse, isLoading, error } = useQuery({
        queryKey: ["market", childMarket.id, "child-full"],
        queryFn: async () => {
            console.log("🚀 Query STARTING for:", childMarket.id);
            try {
                const result = await MarketGetters.getMarketById(childMarket.id);
                console.log("✅ Query SUCCESS for:", childMarket.id, result);
                return result;
            } catch (err) {
                console.error("❌ Query FAILED for:", childMarket.id, err);
                throw err;
            }
        },
        enabled: !!childMarket.id,
    });



    const fullMarket = marketResponse?.market;
    const volumeInfo = marketResponse?.volumeInfo;
    const marketPrice = marketResponse?.marketPrice;
    // Print marketPrice

    // Use latest market price from response, fallback to childMarket data
    const yesPrice = marketPrice?.latestYesPrice ?? childMarket.yesPrice;
    const noPrice = marketPrice?.latestNoPrice ?? childMarket.noPrice;
    const yesPercentage = (yesPrice * 100).toFixed(0);
    const noPercentage = (noPrice * 100).toFixed(0);

    // Calculate volume
    const totalVolume = calculateTotalVolume(volumeInfo);
    const formattedVolume = formatVolume(totalVolume);

    const [firstButtonColors, setFirstButtonColors] = useState<{ primary: string; secondary: string } | null>(null);
    const [secondButtonColors, setSecondButtonColors] = useState<{ primary: string; secondary: string } | null>(null);

    // Check if market is initialized (has real prices)
    const [isMarketInitialized, setIsMarketInitialized] = useState(false);

    useEffect(() => {
        // Use a small epsilon to handle floating point precision
        const EPSILON = 0.0001;
        const isDefaultYes = Math.abs(yesPrice - 0.5) < EPSILON;
        const isDefaultNo = Math.abs(noPrice - 0.5) < EPSILON;
        const hasRealPrices = !(isDefaultYes && isDefaultNo);

        console.log(`📊 ${childMarket.name}:`, {
            yesPrice: yesPrice.toFixed(6),
            noPrice: noPrice.toFixed(6),
            isDefaultYes,
            isDefaultNo,
            hasRealPrices
        });

        setIsMarketInitialized(hasRealPrices);
    }, [yesPrice, noPrice, childMarket.name]);

    useEffect(() => {
        if (childMarket.logo && childMarket.logo.length >= 2) {
            getImageColors(childMarket.logo[0]).then(colors => setFirstButtonColors(colors)).catch(console.error);
            getImageColors(childMarket.logo[1]).then(colors => setSecondButtonColors(colors)).catch(console.error);
        } else {
            setFirstButtonColors(null);
            setSecondButtonColors(null);
        }
    }, [childMarket.logo]);

    // Show loading state while fetching
    if (isLoading) {
        return (
            <Box
                p={3}
                bg={cardBg}
                borderRadius="lg"
                borderWidth="1px"
                borderColor={borderColor}
                h="100%"
            >
                <Center py={4}>
                    <Spinner size="sm" />
                </Center>
            </Box>
        );
    }

    return (
        <Box
            p={3}
            bg={cardBg}
            borderRadius="lg"
            borderWidth="1px"
            borderColor={borderColor}
            cursor="pointer"
            _hover={{ borderColor: "blue.400", bg: useColorModeValue("gray.100", "gray.600") }}
            transition="all 0.2s"
            onClick={() => router.push(`/market/${childMarket.id}`)}
            h="100%"
        >
            <VStack align="stretch" gap={2}>
                <HStack gap={2}>
                    <Avatar.Root size="xs" borderRadius="full">
                        <Avatar.Image
                            boxSize="20px"
                            objectFit="cover"
                            src={childMarket.logo && childMarket.logo.length > 0 ? childMarket.logo[0] : undefined}
                        />
                        <Avatar.Fallback>
                            {childMarket.name.slice(0, 2).toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                    <Text fontSize="sm" fontWeight="medium" lineClamp={1} flex={1}>
                        {childMarket.name}
                    </Text>
                    <Badge
                        colorScheme={getStatusColor(childMarket.status)}
                        variant="solid"
                        size="sm"
                        borderRadius="full"
                        fontSize="2xs"
                        px={2}
                    >
                        {MarketStatus[childMarket.status]}
                    </Badge>
                </HStack>

                <Text fontSize="xs" color="gray.500" lineClamp={2}>
                    {childMarket.description || childMarket.question}
                </Text>

                {/* Volume display */}
                <HStack justify="flex-end">
                    <HStack gap={1} color="gray.500" fontSize="2xs">
                        <Icon boxSize={2}><BarChart3 /></Icon>
                        <Text>Vol {formattedVolume}</Text>
                    </HStack>
                </HStack>

                <HStack gap={2} mt={1}>
                    <Button
                        size="sm"
                        flex={1}
                        py={2}
                        onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            router.push(`/market/${childMarket.id}`);
                        }}
                        bg="transparent"
                        position="relative"
                        overflow="hidden"
                        color={textColor}
                        fontWeight="bold"
                        border="1px solid"
                        borderColor="rgba(255, 255, 255, 0.2)"
                        backdropFilter="blur(8px)"
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
                        }}
                        _hover={{ transform: 'scale(1.02)', _before: { opacity: 0.9 } }}
                        css={{ '& > *': { position: 'relative', zIndex: 1 } }}
                    >
                        <TrendingUp size={12} style={{ marginRight: '4px' }} />
                        Yes {yesPercentage}%
                    </Button>
                    <Button
                        size="sm"
                        flex={1}
                        py={2}
                        onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            router.push(`/market/${childMarket.id}`);
                        }}
                        bg="transparent"
                        position="relative"
                        overflow="hidden"
                        color={textColor}
                        fontWeight="bold"
                        border="1px solid"
                        borderColor="rgba(255, 255, 255, 0.2)"
                        backdropFilter="blur(8px)"
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
                        }}
                        _hover={{ transform: 'scale(1.02)', _before: { opacity: 0.9 } }}
                        css={{ '& > *': { position: 'relative', zIndex: 1 } }}
                    >
                        <Activity size={12} style={{ marginRight: '4px' }} />
                        No {noPercentage}%
                    </Button>
                </HStack>

                {/* Admin buttons - only show if in admin view AND market is OPEN AND initialized */}
                {childMarket.status === MarketStatus.OPEN && isAdminView && (
                    <Box mt={2} pt={1} borderTopWidth="1px" borderTopColor={borderColor}>
                        <AdminMarketButtons
                            market={childMarket}
                            isInitialized={isMarketInitialized}
                        />
                    </Box>
                )}
            </VStack>
        </Box>
    );
};

// Main Event Page Component
export default function EventPage() {
    const params = useParams();
    const router = useRouter();
    const eventId = params.id as string;
    const searchParams = useSearchParams();
    const fromAdmin = searchParams.get('fromAdmin') === 'true';
    const bgColor = useColorModeValue("gray.50", "gray.900");
    const textColor = useColorModeValue("gray.800", "white");

    // Fetch ONLY the parent market (includes child markets already)
    const { data: parentMarketResponse, isLoading: parentLoading } = useQuery({
        queryKey: ["event", eventId],
        queryFn: () => MarketGetters.getMarketById(eventId),
        enabled: !!eventId,
    });

    const parentMarket = parentMarketResponse?.market;
    const parentVolume = parentMarketResponse?.volumeInfo;
    const parentMarketPrice = parentMarketResponse?.marketPrice;

    // Loading state
    if (parentLoading) {
        return (
            <Center minH="100vh" bg={bgColor}>
                <VStack gap={4}>
                    <Spinner size="xl" color="blue.500" />
                    <Text color="gray.500">Loading event details...</Text>
                </VStack>
            </Center>
        );
    }

    // Not found
    if (!parentMarket) {
        return (
            <Center minH="100vh" bg={bgColor}>
                <VStack gap={4}>
                    <Text fontSize="xl" color="gray.500">Event not found</Text>
                    <Link href="/">
                        <Button>Back to Home</Button>
                    </Link>
                </VStack>
            </Center>
        );
    }

    // Calculate parent volume
    const parentTotalVolume = calculateTotalVolume(parentVolume);
    const parentFormattedVolume = formatVolume(parentTotalVolume);

    // Child markets are already in parentMarket.childMarkets - NO EXTRA FETCHING!
    const childMarkets = parentMarket.childMarkets || [];

    return (
        <Box minH="100vh" bg={bgColor} pt={8} pb={16}>
            <Container maxW="7xl">
                {/* Back Button */}
                <Button variant="ghost">
                    <ArrowLeft size={18} className="mr-2" />
                    Back to Home
                </Button>

                {/* Main Event Card with Child Markets */}
                <Box
                    bg={useColorModeValue("white", "gray.800")}
                    borderRadius="xl"
                    overflow="hidden"
                    borderWidth="1px"
                    borderColor={useColorModeValue("gray.200", "gray.700")}
                >
                    <Box p={5}>
                        <VStack align="stretch" gap={4}>
                            {/* Header: Name + Badge + Time */}
                            <Flex justify="space-between" align="start">
                                <HStack gap={3} flex={1}>
                                    <Avatar.Root size="md" borderRadius="full">
                                        <Avatar.Image
                                            boxSize="40px"
                                            objectFit="cover"
                                            src={parentMarket.logo && parentMarket.logo.length > 0 ? parentMarket.logo[0] : undefined}
                                            alt={parentMarket.name}
                                        />
                                        <Avatar.Fallback>
                                            {parentMarket.name.slice(0, 2).toUpperCase()}
                                        </Avatar.Fallback>
                                    </Avatar.Root>
                                    <Heading size="md" lineClamp={1}>
                                        {parentMarket.name}
                                    </Heading>
                                </HStack>
                                <HStack gap={2}>
                                    <Badge colorScheme="purple" borderRadius="full">
                                        Event
                                    </Badge>
                                    <Badge
                                        colorScheme={getStatusColor(parentMarket.status)}
                                        variant="solid"
                                        size="sm"
                                        borderRadius="full"
                                    >
                                        {MarketStatus[parentMarket.status]}
                                    </Badge>
                                </HStack>
                            </Flex>

                            {/* Description */}
                            {parentMarket.description && (
                                <Text fontSize="sm" color="gray.500" lineClamp={2}>
                                    {parentMarket.description}
                                </Text>
                            )}

                            {/* Stats Row: Time Left + Volume + Child Count */}
                            <HStack gap={4} wrap="wrap">
                                <HStack gap={1}>
                                    <Icon boxSize={4}><Clock size={14} /></Icon>
                                    <Text fontSize="sm" fontWeight="medium">
                                        {getTimeRemaining(parentMarket.marketExpiry)} left
                                    </Text>
                                </HStack>
                                <HStack gap={1}>
                                    <Icon boxSize={4}><BarChart3 size={14} /></Icon>
                                    <Text fontSize="sm" fontWeight="medium">
                                        Vol {parentFormattedVolume}
                                    </Text>
                                </HStack>
                                <HStack gap={1}>
                                    <Icon boxSize={4}><Users size={14} /></Icon>
                                    <Text fontSize="sm" fontWeight="medium">
                                        {childMarkets.length} Markets
                                    </Text>
                                </HStack>
                            </HStack>

                            {/* Resolution Criteria */}
                            {parentMarket.resolutionCriteria && (
                                <Box
                                    bg={useColorModeValue("gray.50", "gray.700")}
                                    p={2}
                                    px={3}
                                    borderRadius="md"
                                >
                                    <Text fontSize="xs" fontWeight="bold" color="gray.500">
                                        Resolution Criteria:
                                    </Text>
                                    <Text fontSize="xs" color={textColor} lineClamp={2}>
                                        {parentMarket.resolutionCriteria}
                                    </Text>
                                </Box>
                            )}

                            {/* Separator before Child Markets */}
                            <Separator my={2} />


                            {/* Child Markets Section - Inside same box */}
                            <Box>
                                <HStack justify="space-between" mb={3}>
                                    <Heading size="sm">
                                        Child Markets
                                    </Heading>
                                    <Text fontSize="xs" color="gray.500">
                                        {childMarkets.length} markets
                                    </Text>
                                </HStack>

                                {childMarkets.length === 0 ? (
                                    <Center py={4}>
                                        <Text fontSize="sm" color="gray.500">No child markets available.</Text>
                                    </Center>
                                ) : (
                                    <SimpleGrid columns={{ base: 1, md: 2, lg: 3 }} gap={3}>
                                        {childMarkets.map((childMarket) => (
                                            <ChildMarketCard key={childMarket.id} childMarket={childMarket} isAdminView={fromAdmin} />
                                        ))}
                                    </SimpleGrid>
                                )}
                            </Box>
                        </VStack>
                    </Box>
                </Box>
            </Container>
        </Box>
    );
}
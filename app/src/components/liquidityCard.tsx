

"use client";

import { Card, VStack, Text, HStack, Badge, Flex, Avatar, Heading, Box, Dialog, Button, Input, NumberInput, Field } from "@chakra-ui/react";
import { useState, useRef, useEffect } from "react";
import { useColorModeValue } from "@/components/ui/color-mode";
import { Market, MarketStatus } from "@/generated/grpc_service_types/markets";
import { Clock, X, Wallet } from "lucide-react";
import { LiquidityService } from "@/utils/interactions/dataPosters";
import { toaster } from "./ui/toaster";

interface Props {
    market: Market;
    onDepositLiquidity?: (marketId: string, amount: number) => void;
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

// Format liquidity pool size
function formatLiquidity(liquidityB: number): string {
    if (liquidityB >= 1_000_000) {
        return `${(liquidityB / 1_000_000).toFixed(1)}M KES`;
    } else if (liquidityB >= 1_000) {
        return `${(liquidityB / 1_000).toFixed(1)}K KES`;
    } else {
        return `${liquidityB} KES`;
    }
}

const TrendingliquidityMarketCard = ({ market, onDepositLiquidity }: Props) => {
    const [isDialogOpen, setIsDialogOpen] = useState(false);
    const [depositAmount, setDepositAmount] = useState<string>("");
    const [isDepositing, setIsDepositing] = useState(false);
    const pressTimer = useRef<NodeJS.Timeout | null>(null);
    const cardBg = useColorModeValue("white", "gray.800");
    const borderColor = useColorModeValue("gray.200", "gray.600");
    const textColor = useColorModeValue("gray.800", "white");
    const timeRemaining = getTimeRemaining(market.marketExpiry);

    // Get liquidity pool size (liquidity_b)
    const liquidityPool = market.liquidityB || 0;
    const formattedLiquidity = formatLiquidity(liquidityPool);

    // Handle mouse/touch start (long press)
    const handlePressStart = () => {
        pressTimer.current = setTimeout(() => {
            setIsDialogOpen(true);
        }, 500); // 500ms long press
    };

    // Handle mouse/touch end (cancel long press)
    const handlePressEnd = () => {
        if (pressTimer.current) {
            clearTimeout(pressTimer.current);
            pressTimer.current = null;
        }
    };

    // Clean up timer on unmount
    useEffect(() => {
        return () => {
            if (pressTimer.current) {
                clearTimeout(pressTimer.current);
            }
        };
    }, []);




    const handleDeposit = async () => {
        const amount = parseFloat(depositAmount);
        if (isNaN(amount) || amount <= 0) {
            toaster.error({
                title: "Invalid amount",
                description: "Please enter a valid amount greater than 0",
            });
            return;
        }

        if (amount < 100) {
            toaster.error({
                title: "Minimum deposit",
                description: "Minimum deposit amount is 100 KES",
            });
            return;
        }

        toaster.promise(
            LiquidityService.addLiquidity({
                market_id: market.id,
                amount: amount,
            }),
            {
                loading: {
                    title: "Adding liquidity...",
                    description: `Processing ${amount} KES deposit`,
                },
                success: (position) => {
                    // Close dialog on success
                    setIsDialogOpen(false);
                    setDepositAmount("");

                    return {
                        title: "Liquidity added successfully!",
                        description: `Successfully added ${amount} KES to the pool. You own ${(position.shares_of_pool * 100).toFixed(2)}% of the pool.`,
                    };
                },
                error: (error: any) => ({
                    title: "Deposit failed",
                    description: error?.message || "Failed to add liquidity. Please try again.",
                }),
            }
        );
    };



    return (
        <>
            {/* Card with long press handler */}
            <Card.Root
                width="320px"
                minH="160px"
                bg={cardBg}
                borderColor={borderColor}
                borderWidth="1px"
                borderRadius="xl"
                cursor="pointer"
                transition="all 0.2s"
                overflow="hidden"
                _hover={{ shadow: "md", transform: "translateY(-2px)" }}
                onClick={() => setIsDialogOpen(true)}
            >
                <Card.Body p={4}>
                    <VStack gap={3} align="stretch">

                        {/* Header: Avatar + Status */}
                        <Flex justify="space-between" align="start">
                            <HStack gap={3}>
                                <Avatar.Root size="sm" borderRadius="full">
                                    <Avatar.Image
                                        boxSize="32px"
                                        objectFit="cover"
                                        src={market.logo && market.logo.length > 0 ? market.logo[0] : undefined}
                                        alt={market.name}
                                    />
                                    <Avatar.Fallback fontSize="xs">
                                        {market.name.includes(' ')
                                            ? market.name
                                                .split(' ')
                                                .map(word => word[0])
                                                .join('')
                                                .toUpperCase()
                                                .slice(0, 2)
                                            : market.name.slice(0, 2).toUpperCase()
                                        }
                                    </Avatar.Fallback>
                                </Avatar.Root>

                                <VStack align="start" gap={0}>
                                    <HStack gap={1} color="gray.500">
                                        <Clock size={12} />
                                        <Text fontSize="xs">{timeRemaining} left</Text>
                                    </HStack>
                                </VStack>
                            </HStack>

                            <Badge
                                colorScheme={getStatusColor(market.status)}
                                variant="solid"
                                size="sm"
                                borderRadius="full"
                                px={2}
                            >
                                {MarketStatus[market.status]}
                            </Badge>
                        </Flex>

                        {/* Market Name */}
                        <Heading
                            size="sm"
                            lineHeight="1.3"
                            lineClamp={2}
                            color={textColor}
                            fontWeight="semibold"
                        >
                            {market.name}
                        </Heading>

                        {/* Liquidity Pool Size */}
                        <Box>
                            <HStack justify="space-between" align="baseline">
                                <Text fontSize="xs" color="gray.500">
                                    Pool Size
                                </Text>
                                <Text fontSize="md" fontWeight="bold" color="blue.500">
                                    {formattedLiquidity}
                                </Text>
                            </HStack>
                        </Box>

                        {/* Deposit hint */}
                        <Text fontSize="xs" color="gray.400" textAlign="center">
                            Long press to deposit liquidity
                        </Text>

                    </VStack>
                </Card.Body>
            </Card.Root>

            {/* Deposit Dialog */}
            <Dialog.Root
                open={isDialogOpen}
                onOpenChange={(details) => setIsDialogOpen(details.open)}
                placement="center"
            >
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content
                        position="fixed"
                        bottom={0}
                        left={0}
                        right={0}
                        margin={0}
                        width="100%"
                        maxWidth="100%"
                        borderRadius="xl"
                        borderBottomRadius={0}
                        bg={cardBg}
                        boxShadow="0 -4px 20px rgba(0,0,0,0.15)"
                        _open={{
                            animationName: "slide-up",
                            animationDuration: "0.3s",
                            animationTimingFunction: "ease",
                        }}
                        _closed={{
                            animationName: "slide-down",
                            animationDuration: "0.3s",
                            animationTimingFunction: "ease",
                        }}
                        css={{
                            "@keyframes slide-up": {
                                from: { transform: "translateY(100%)" },
                                to: { transform: "translateY(0)" },
                            },
                            "@keyframes slide-down": {
                                from: { transform: "translateY(0)" },
                                to: { transform: "translateY(100%)" },
                            },
                        }}
                    >
                        <Dialog.Header borderBottom="none" pb={0}>
                            <Flex justify="space-between" align="center" w="full">
                                <Dialog.Title fontSize="lg" fontWeight="bold">
                                    Add Liquidity
                                </Dialog.Title>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => setIsDialogOpen(false)}
                                    p={0}
                                    minW="auto"
                                >
                                    <X size={20} />
                                </Button>
                            </Flex>
                        </Dialog.Header>

                        <Dialog.Body pb={4}>
                            <VStack gap={4} align="stretch">
                                {/* Market info summary */}
                                <Flex justify="space-between" align="center">
                                    <HStack gap={4}>
                                        <Avatar.Root size="md" borderRadius="full">
                                            <Avatar.Image
                                                boxSize="40px"
                                                objectFit="cover"
                                                src={market.logo && market.logo.length > 0 ? market.logo[0] : undefined}
                                                alt={market.name}
                                            />
                                        </Avatar.Root>
                                        <Box>
                                            <Text fontSize="sm" fontWeight="medium">{market.name}</Text>
                                            <Text fontSize="xs" color="gray.500">
                                                Current Pool: {formattedLiquidity}
                                            </Text>
                                        </Box>
                                    </HStack>
                                </Flex>

                                {/* Amount Input */}
                                <Field.Root>
                                    <Field.Label fontSize="sm" fontWeight="medium">
                                        Amount (KES)
                                    </Field.Label>
                                    <NumberInput.Root
                                        value={depositAmount}
                                        onValueChange={(e) => setDepositAmount(e.value)}
                                        min={0}
                                        step={100}
                                    >
                                        <NumberInput.Control />
                                        <NumberInput.Input />
                                    </NumberInput.Root>
                                    <Field.HelperText fontSize="xs" color="gray.500">
                                        Minimum deposit: 100 KES
                                    </Field.HelperText>
                                </Field.Root>

                                {/* Estimated share */}
                                {depositAmount && parseFloat(depositAmount) > 0 && (
                                    <Box bg="gray.50" p={3} borderRadius="md">
                                        <Text fontSize="sm" fontWeight="medium">Estimated Pool Share</Text>
                                        <Text fontSize="lg" fontWeight="bold" color="blue.500">
                                            {((parseFloat(depositAmount) / (liquidityPool + parseFloat(depositAmount))) * 100).toFixed(2)}%
                                        </Text>
                                        <Text fontSize="xs" color="gray.500">
                                            You will own this percentage of the liquidity pool
                                        </Text>
                                    </Box>
                                )}

                                {/* Deposit Button */}
                                <Button
                                    colorScheme="blue"
                                    size="lg"
                                    onClick={handleDeposit}
                                    loading={isDepositing}
                                    disabled={!depositAmount || parseFloat(depositAmount) <= 0}
                                >
                                    <Wallet size={18} style={{ marginRight: '8px' }} />
                                    Deposit Liquidity
                                </Button>
                            </VStack>
                        </Dialog.Body>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Dialog.Root>
        </>
    );
};

export default TrendingliquidityMarketCard;
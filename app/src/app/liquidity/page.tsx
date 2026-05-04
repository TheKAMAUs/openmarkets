"use client";

import { Container, VStack, Text, Box, HStack, SimpleGrid, Spinner, Center, Tabs, Table, Badge, Button, Dialog, Portal, NumberInput, Field } from "@chakra-ui/react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { MarketGetters } from "@/utils/interactions/dataGetter";
import { MarketStatus } from "@/generated/grpc_service_types/markets";
import TrendingMarketCard from "@/components/TrendingMarketCard";
import TrendingliquidityMarketCard from "@/components/liquidityCard";
import { LiquidityService } from "@/utils/interactions/dataPosters";

import useUserInfo from "@/hooks/useUserInfo";
import { toaster } from "@/components/ui/toaster";





export default function OpenMarketsPage() {
    const [activeTab, setActiveTab] = useState<"markets" | "positions">("markets");
    const { data: user } = useUserInfo();
    const queryClient = useQueryClient();

    // Withdraw dialog state
    const [withdrawDialogOpen, setWithdrawDialogOpen] = useState(false);
    const [selectedPosition, setSelectedPosition] = useState<any>(null);
    const [withdrawAmount, setWithdrawAmount] = useState<string>("");
    const [isWithdrawing, setIsWithdrawing] = useState(false);

    // Fetch open markets
    const { data: markets = [], isLoading: marketsLoading, error: marketsError } = useQuery({
        queryKey: ["openMarkets", 1, 100, "open"],
        queryFn: () => MarketGetters.getMarketData(1, 100, MarketStatus.OPEN),
        enabled: activeTab === "markets",
    });

    // Fetch user's fee earnings
    const { data: feeData, isLoading: feesLoading, refetch: refetchFees } = useQuery({
        queryKey: ["feeEarnings", user?.public_key],
        queryFn: () => LiquidityService.getFeeEarnings(),
        enabled: !!user && activeTab === "positions",
    });

    const totalFees = feeData?.total_fees_earned || 0;
    const positions = feeData?.positions || [];
    console.log("📊 Positions data:", positions);

    // Filter out child markets
    const parentOpenMarkets = useMemo(() => {
        return markets.filter(market => !market.parentId || market.parentId === "");
    }, [markets]);

    // Format currency
    const formatCurrency = (value: number) => {
        return new Intl.NumberFormat('en-KE', {
            style: 'currency',
            currency: 'KES',
            minimumFractionDigits: 3,
            maximumFractionDigits: 3,
        }).format(value);
    };

    // Format percentage
    const formatPercentage = (value: number) => {
        return `${(value * 100).toFixed(2)}%`;
    };

    // Handle withdraw click
    const handleWithdrawClick = (position: any) => {
        setSelectedPosition(position);
        setWithdrawAmount("");
        setWithdrawDialogOpen(true);
    };


    // Handle withdraw confirmation
    const handleWithdraw = async () => {
        // Early return if no position selected
        if (!selectedPosition) {
            toaster.error({
                title: "Error",
                description: "No position selected",
            });
            return;
        }

        const amount = parseFloat(withdrawAmount);

        if (isNaN(amount) || amount <= 0) {
            toaster.error({
                title: "Invalid amount",
                description: "Please enter a valid amount greater than 0",
            });
            return;
        }

        if (amount > selectedPosition.amount_deposited) {
            toaster.error({
                title: "Insufficient balance",
                description: `Maximum withdraw amount is ${formatCurrency(selectedPosition.amount_deposited)}`,
            });
            return;
        }

        setIsWithdrawing(true);

        try {
            await LiquidityService.removeLiquidity({
                lp_position_id: selectedPosition.lp_position_id,
                withdraw_amount: amount,
            });

            toaster.success({
                title: "Withdrawal successful!",
                description: `Successfully withdrew ${formatCurrency(amount)} from liquidity pool`,
            });

            setWithdrawDialogOpen(false);
            setSelectedPosition(null);
            setWithdrawAmount("");
            refetchFees();

        } catch (error: any) {
            toaster.error({
                title: "Withdrawal failed",
                description: error?.message || "Failed to withdraw liquidity. Please try again.",
            });
        } finally {
            setIsWithdrawing(false);
        }
    };





    if (activeTab === "markets" && marketsLoading) {
        return (
            <Center h="50vh">
                <Spinner size="xl" color="blue.500" />
                <Text ml={3}>Loading markets...</Text>
            </Center>
        );
    }

    if (activeTab === "markets" && marketsError) {
        return (
            <Center h="50vh">
                <Text color="red.500">Error loading markets: {marketsError.message}</Text>
            </Center>
        );
    }

    if (activeTab === "positions" && feesLoading) {
        return (
            <Center h="50vh">
                <Spinner size="xl" color="blue.500" />
                <Text ml={3}>Loading your positions...</Text>
            </Center>
        );
    }

    return (
        <Container maxW="7xl" py={10}>
            <Tabs.Root
                value={activeTab}
                onValueChange={(e) => setActiveTab(e.value as "markets" | "positions")}
                variant="enclosed"
            >
                <Tabs.List mb={6}>
                    <Tabs.Trigger value="markets">
                        <Text fontSize="lg" fontWeight="medium">Markets</Text>
                    </Tabs.Trigger>
                    <Tabs.Trigger value="positions">
                        <Text fontSize="lg" fontWeight="medium">My Positions</Text>
                    </Tabs.Trigger>
                </Tabs.List>

                {/* Markets Tab */}
                <Tabs.Content value="markets">
                    <VStack align="start" gap={8}>
                        <Box w="full">
                            <Text fontSize="3xl" fontWeight="bold" mb={2}>
                                Open Markets
                            </Text>
                            <Text fontSize="md" color="gray.500">
                                {parentOpenMarkets.length} active markets available for trading
                            </Text>
                        </Box>

                        {parentOpenMarkets.length > 0 ? (
                            <SimpleGrid columns={{ base: 1, md: 2, lg: 3 }} gap={6} w="full">
                                {parentOpenMarkets.map((market) => (
                                    <TrendingliquidityMarketCard key={market.id} market={market} />
                                ))}
                            </SimpleGrid>
                        ) : (
                            <Box w="full" textAlign="center" py={10}>
                                <Text color="gray.500">No open markets available at the moment.</Text>
                            </Box>
                        )}
                    </VStack>
                </Tabs.Content>

                {/* My Positions Tab */}
                <Tabs.Content value="positions">
                    <VStack align="start" gap={8}>
                        <Box w="full">
                            <Text fontSize="3xl" fontWeight="bold" mb={2}>
                                My Liquidity Positions
                            </Text>
                            <Text fontSize="md" color="gray.500">
                                Manage your liquidity across different markets
                            </Text>
                        </Box>

                        {!user ? (
                            <Box w="full" textAlign="center" py={10}>
                                <Text color="gray.500">Please login to view your positions</Text>
                            </Box>
                        ) : positions.length > 0 ? (



                            <Box w="full" overflowX="auto">
                                <Table.Root variant="outline" striped>
                                    <Table.Header>
                                        <Table.Row>
                                            <Table.ColumnHeader>Market</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Amount Deposited</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Pool Share</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Fees Earned</Table.ColumnHeader>
                                            <Table.ColumnHeader>Status</Table.ColumnHeader>
                                            <Table.ColumnHeader>Actions</Table.ColumnHeader>
                                        </Table.Row>
                                    </Table.Header>
                                    <Table.Body>
                                        {positions.map((position) => {
                                            console.log("📊 Position in map:", position);
                                            console.log("📊 shares_of_pool:", position.shares_of_pool);

                                            console.log("📊 fees_earned:", position.fees_earned);
                                            return (
                                                <Table.Row key={position.lp_position_id}>
                                                    <Table.Cell fontWeight="medium">
                                                        {position.market_id.slice(0, 8)}...
                                                    </Table.Cell>
                                                    <Table.Cell textAlign="right">{formatCurrency(position.amount_deposited)}</Table.Cell>
                                                    <Table.Cell textAlign="right">{formatPercentage(position.shares_of_pool)}</Table.Cell>
                                                    <Table.Cell textAlign="right">{formatCurrency(position.fees_earned)}</Table.Cell>
                                                    <Table.Cell>
                                                        <Badge colorScheme={position.is_active ? "green" : "gray"}>
                                                            {position.is_active ? "Active" : "Inactive"}
                                                        </Badge>
                                                    </Table.Cell>
                                                    <Table.Cell>
                                                        <Button
                                                            size="sm"
                                                            colorScheme="red"
                                                            variant="outline"
                                                            onClick={() => handleWithdrawClick(position)}
                                                        >
                                                            Withdraw
                                                        </Button>
                                                    </Table.Cell>
                                                </Table.Row>
                                            );
                                        })}
                                    </Table.Body>
                                </Table.Root>
                            </Box>
                        ) : (
                            <Box w="full" textAlign="center" py={10}>
                                <Text color="gray.500">You don't have any liquidity positions yet.</Text>
                                <Text fontSize="sm" color="gray.400" mt={2}>
                                    Go to the Markets tab and long-press on a market card to add liquidity.
                                </Text>
                            </Box>
                        )}
                    </VStack>
                </Tabs.Content>
            </Tabs.Root>

            {/* Withdraw Dialog */}
            <Dialog.Root open={withdrawDialogOpen} onOpenChange={(details) => setWithdrawDialogOpen(details.open)} placement="center">
                <Portal>
                    <Dialog.Backdrop />
                    <Dialog.Positioner>
                        <Dialog.Content width="90%" maxWidth="400px">
                            <Dialog.Header>
                                <Dialog.Title>Withdraw Liquidity</Dialog.Title>
                            </Dialog.Header>
                            <Dialog.Body>
                                <VStack gap={4}>
                                    <Box w="full">
                                        <Text fontSize="sm" fontWeight="medium" mb={2}>
                                            Position ID
                                        </Text>
                                        <Text fontSize="sm" color="gray.500">
                                            {selectedPosition?.lp_position_id?.slice(0, 8)}...
                                        </Text>
                                    </Box>
                                    <Box w="full">
                                        <Text fontSize="sm" fontWeight="medium" mb={2}>
                                            Available to withdraw
                                        </Text>
                                        <Text fontSize="lg" fontWeight="bold" color="blue.500">
                                            {formatCurrency(selectedPosition?.amount_deposited || 0)}
                                        </Text>
                                    </Box>
                                    <Field.Root>
                                        <Field.Label>Withdraw Amount (KES)</Field.Label>
                                        <NumberInput.Root
                                            value={withdrawAmount}
                                            onValueChange={(e) => setWithdrawAmount(e.value)}
                                            min={0}
                                            max={selectedPosition?.amount_deposited || 0}
                                            step={100}
                                        >
                                            <NumberInput.Control />
                                            <NumberInput.Input />
                                        </NumberInput.Root>
                                        <Field.HelperText>
                                            Minimum: 100 KES | Maximum: {formatCurrency(selectedPosition?.amount_deposited || 0)}
                                        </Field.HelperText>
                                    </Field.Root>
                                </VStack>
                            </Dialog.Body>
                            <Dialog.Footer>
                                <Button
                                    variant="outline"
                                    onClick={() => setWithdrawDialogOpen(false)}
                                >
                                    Cancel
                                </Button>
                                <Button
                                    colorScheme="red"
                                    onClick={handleWithdraw}
                                    loading={isWithdrawing}
                                    disabled={!withdrawAmount || parseFloat(withdrawAmount) <= 0}
                                >
                                    Withdraw
                                </Button>
                            </Dialog.Footer>
                        </Dialog.Content>
                    </Dialog.Positioner>
                </Portal>
            </Dialog.Root>
        </Container>
    );
}
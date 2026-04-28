// app/leaderboard/page.tsx
"use client";

import {
    Container,
    VStack,
    HStack,
    Heading,
    Text,
    Box,
    Avatar,
    Badge,
    Table,
    Tabs,
    Spinner,
    Center,

    Card,
    SimpleGrid,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import { Trophy, TrendingUp, TrendingDown, Users } from "lucide-react";
import { ProfitLossAPI, UserProfitRanking } from "@/utils/interactions/dataGetter";
import { useColorModeValue } from "@/components/ui/color-mode";

// Helper to format profit
const formatProfit = (profit: number) => {
    const absProfit = Math.abs(profit);
    const formatted = `$${absProfit.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
    return profit >= 0 ? `+${formatted}` : `-${formatted}`;
};

// Helper to get profit color
const getProfitColor = (profit: number) => {
    if (profit > 0) return "green.500";
    if (profit < 0) return "red.500";
    return "gray.500";
};

// Rank Medal Component
const RankMedal = ({ rank }: { rank: number }) => {
    if (rank === 1) {
        return <Badge colorScheme="yellow" borderRadius="full" px={2}>🥇 #1</Badge>;
    }
    if (rank === 2) {
        return <Badge colorScheme="gray" borderRadius="full" px={2}>🥈 #2</Badge>;
    }
    if (rank === 3) {
        return <Badge colorScheme="orange" borderRadius="full" px={2}>🥉 #3</Badge>;
    }
    return <Badge colorScheme="blue" borderRadius="full" px={2}>#{rank}</Badge>;
};

// User Row Component
const UserRow = ({ user, isWinner }: { user: UserProfitRanking; isWinner: boolean }) => {
    const bgHover = useColorModeValue("gray.50", "gray.700");
    const profitColor = getProfitColor(user.net_profit);
    const profitFormatted = formatProfit(user.net_profit);
    const winRate = (user.win_rate * 100).toFixed(1);

    return (
        <Table.Row _hover={{ bg: bgHover }} transition="all 0.2s">
            <Table.Cell fontWeight="bold" width="80px">
                <RankMedal rank={user.rank} />
            </Table.Cell>
            <Table.Cell>
                <HStack gap={3}>
                    <Avatar.Root size="sm" borderRadius="full">
                        <Avatar.Image src={user.avatar || undefined} alt={user.name} />
                        <Avatar.Fallback>
                            {user.name.slice(0, 2).toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                    <VStack align="start" gap={0}>
                        <Text fontWeight="medium">{user.name}</Text>
                        <Text fontSize="xs" color="gray.500">{user.email}</Text>
                    </VStack>
                </HStack>
            </Table.Cell>
            <Table.Cell textAlign="right">
                <Text fontWeight="bold" color={profitColor}>
                    {profitFormatted}
                </Text>
            </Table.Cell>
            <Table.Cell textAlign="right">
                <HStack gap={2} justify="flex-end">
                    <Badge colorScheme="green" borderRadius="full" px={2}>
                        {user.winning_trades}W
                    </Badge>
                    <Badge colorScheme="red" borderRadius="full" px={2}>
                        {user.losing_trades}L
                    </Badge>
                </HStack>
            </Table.Cell>
            <Table.Cell textAlign="right">
                <Text fontWeight="medium">{winRate}%</Text>
            </Table.Cell>
            <Table.Cell textAlign="right">
                <Text>{user.total_trades}</Text>
            </Table.Cell>
        </Table.Row>
    );
};

// Stats Card Component
const StatsCard = ({ title, value, icon: Icon, color }: { title: string; value: string | number; icon: any; color: string }) => {
    const bgColor = useColorModeValue("white", "gray.800");

    return (
        <Card.Root bg={bgColor} borderRadius="xl" boxShadow="sm">
            <Card.Body>
                <HStack justify="space-between">
                    <VStack align="start" gap={1}>
                        <Text fontSize="sm" color="gray.500">{title}</Text>
                        <Text fontSize="2xl" fontWeight="bold" color={color}>
                            {value}
                        </Text>
                    </VStack>
                    <Box p={2} bg={`${color}.100`} borderRadius="lg">
                        <Icon size={24} color={color} />
                    </Box>
                </HStack>
            </Card.Body>
        </Card.Root>
    );
};

export default function LeaderboardPage() {
    const bgColor = useColorModeValue("gray.50", "gray.900");
    const textColor = useColorModeValue("gray.800", "white");
    const borderColor = useColorModeValue("gray.200", "gray.700");

    const { data, isLoading, error } = useQuery({
        queryKey: ["leaderboard"],
        queryFn: ProfitLossAPI.getLeaderboard,
    });

    const winners = data?.winners || [];
    const losers = data?.losers || [];

    // Calculate stats
    const totalWinners = winners.length;
    const totalLosers = losers.length;
    const topWinner = winners[0];
    const biggestLoser = losers[0];
    const totalProfit = winners.reduce((sum, w) => sum + w.net_profit, 0);
    const totalLoss = losers.reduce((sum, l) => sum + l.net_profit, 0);

    if (isLoading) {
        return (
            <Center minH="100vh" bg={bgColor}>
                <VStack gap={4}>
                    <Spinner size="xl" color="blue.500" />
                    <Text color="gray.500">Loading leaderboard...</Text>
                </VStack>
            </Center>
        );
    }

    if (error) {
        return (
            <Center minH="100vh" bg={bgColor}>
                <VStack gap={4}>
                    <Text color="red.500">Failed to load leaderboard</Text>
                    <Text fontSize="sm" color="gray.500">{error.message}</Text>
                </VStack>
            </Center>
        );
    }

    return (
        <Box minH="100vh" bg={bgColor} pt={8} pb={16}>
            <Container maxW="7xl">
                {/* Header */}
                <VStack align="start" gap={2} mb={8}>
                    <HStack gap={2}>
                        <Trophy size={32} color="#EAB308" />
                        <Heading size="xl" color={textColor}>
                            Leaderboard
                        </Heading>
                    </HStack>
                    <Text color="gray.500">
                        Top traders ranked by net profit. See who's winning and who's losing.
                    </Text>
                </VStack>

                {/* Stats Grid */}
                <SimpleGrid columns={{ base: 1, md: 2, lg: 4 }} gap={4} mb={8}>
                    <StatsCard
                        title="Total Traders"
                        value={totalWinners + totalLosers}
                        icon={Users}
                        color="blue"
                    />
                    <StatsCard
                        title="Winners"
                        value={totalWinners}
                        icon={TrendingUp}
                        color="green"
                    />
                    <StatsCard
                        title="Losers"
                        value={totalLosers}
                        icon={TrendingDown}
                        color="red"
                    />
                    <StatsCard
                        title="Total P&L"
                        value={formatProfit(totalProfit + totalLoss)}
                        icon={Trophy}
                        color="purple"
                    />
                </SimpleGrid>

                {/* Top Performers */}
                {(topWinner || biggestLoser) && (
                    <SimpleGrid columns={{ base: 1, lg: 2 }} gap={6} mb={8}>
                        {topWinner && (
                            <Card.Root bg={useColorModeValue("white", "gray.800")} borderRadius="xl" borderWidth="1px" borderColor={borderColor}>
                                <Card.Body>
                                    <HStack gap={4}>
                                        <Box p={3} bg="green.100" borderRadius="xl">
                                            <Trophy size={32} color="#22C55E" />
                                        </Box>
                                        <VStack align="start" gap={1}>
                                            <Text fontSize="sm" color="gray.500">🏆 Top Winner</Text>
                                            <HStack gap={2}>
                                                <Avatar.Root size="md">
                                                    <Avatar.Image src={topWinner.avatar || undefined} />
                                                    <Avatar.Fallback>{topWinner.name.slice(0, 2).toUpperCase()}</Avatar.Fallback>
                                                </Avatar.Root>
                                                <Box>
                                                    <Text fontWeight="bold" fontSize="lg">{topWinner.name}</Text>
                                                    <Text fontSize="lg" fontWeight="bold" color="green.500">
                                                        +${topWinner.net_profit.toLocaleString()}
                                                    </Text>
                                                </Box>
                                            </HStack>
                                            <HStack gap={2} mt={2}>
                                                <Badge colorScheme="green">W: {topWinner.winning_trades}</Badge>
                                                <Badge colorScheme="red">L: {topWinner.losing_trades}</Badge>
                                                <Badge>Win Rate: {(topWinner.win_rate * 100).toFixed(1)}%</Badge>
                                            </HStack>
                                        </VStack>
                                    </HStack>
                                </Card.Body>
                            </Card.Root>
                        )}

                        {biggestLoser && (
                            <Card.Root bg={useColorModeValue("white", "gray.800")} borderRadius="xl" borderWidth="1px" borderColor={borderColor}>
                                <Card.Body>
                                    <HStack gap={4}>
                                        <Box p={3} bg="red.100" borderRadius="xl">
                                            <TrendingDown size={32} color="#EF4444" />
                                        </Box>
                                        <VStack align="start" gap={1}>
                                            <Text fontSize="sm" color="gray.500">📉 Biggest Loser</Text>
                                            <HStack gap={2}>
                                                <Avatar.Root size="md">
                                                    <Avatar.Image src={biggestLoser.avatar || undefined} />
                                                    <Avatar.Fallback>{biggestLoser.name.slice(0, 2).toUpperCase()}</Avatar.Fallback>
                                                </Avatar.Root>
                                                <Box>
                                                    <Text fontWeight="bold" fontSize="lg">{biggestLoser.name}</Text>
                                                    <Text fontSize="lg" fontWeight="bold" color="red.500">
                                                        {formatProfit(biggestLoser.net_profit)}
                                                    </Text>
                                                </Box>
                                            </HStack>
                                            <HStack gap={2} mt={2}>
                                                <Badge colorScheme="green">W: {biggestLoser.winning_trades}</Badge>
                                                <Badge colorScheme="red">L: {biggestLoser.losing_trades}</Badge>
                                                <Badge>Win Rate: {(biggestLoser.win_rate * 100).toFixed(1)}%</Badge>
                                            </HStack>
                                        </VStack>
                                    </HStack>
                                </Card.Body>
                            </Card.Root>
                        )}
                    </SimpleGrid>
                )}

                {/* Leaderboard Tables */}
                <Tabs.Root variant="enclosed" colorScheme="blue">
                    <Tabs.List>
                        <Tabs.Trigger value="winners">
                            <HStack gap={2}>
                                <TrendingUp size={16} />
                                <Text>Winners ({totalWinners})</Text>
                            </HStack>
                        </Tabs.Trigger>
                        <Tabs.Trigger value="losers">
                            <HStack gap={2}>
                                <TrendingDown size={16} />
                                <Text>Losers ({totalLosers})</Text>
                            </HStack>
                        </Tabs.Trigger>
                    </Tabs.List>

                    <Tabs.Content value="winners" pt={4} px={0}>
                        {winners.length === 0 ? (
                            <Center py={10}>
                                <Text color="gray.500">No winners yet</Text>
                            </Center>
                        ) : (
                            <Box overflowX="auto">
                                <Table.Root variant="line">

                                    <Table.Header>
                                        <Table.Row>
                                            <Table.ColumnHeader>Rank</Table.ColumnHeader>
                                            <Table.ColumnHeader>Trader</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Net Profit</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">W/L</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Win Rate</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Total Trades</Table.ColumnHeader>
                                        </Table.Row>
                                    </Table.Header>
                                    <Table.Body>
                                        {winners.map((user) => (
                                            <UserRow key={user.user_id} user={user} isWinner={true} />
                                        ))}
                                    </Table.Body>
                                </Table.Root>
                            </Box>
                        )}
                    </Tabs.Content>

                    <Tabs.Content value="losers" pt={4} px={0}>
                        {losers.length === 0 ? (
                            <Center py={10}>
                                <Text color="gray.500">No losers yet</Text>
                            </Center>
                        ) : (
                            <Box overflowX="auto">
                                <Table.Root variant="line">

                                    <Table.Header>
                                        <Table.Row>
                                            <Table.ColumnHeader>Rank</Table.ColumnHeader>
                                            <Table.ColumnHeader>Trader</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Net Profit</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">W/L</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Win Rate</Table.ColumnHeader>
                                            <Table.ColumnHeader textAlign="right">Total Trades</Table.ColumnHeader>
                                        </Table.Row>
                                    </Table.Header>
                                    <Table.Body>
                                        {losers.map((user) => (
                                            <UserRow key={user.user_id} user={user} isWinner={false} />
                                        ))}
                                    </Table.Body>
                                </Table.Root>
                            </Box>
                        )}
                    </Tabs.Content>
                </Tabs.Root>
            </Container>
        </Box>
    );
}
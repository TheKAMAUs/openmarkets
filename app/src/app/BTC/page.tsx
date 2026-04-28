"use client";

import { Container, HStack, VStack, Text, Box, Button, Wrap, WrapItem, Link, Separator, Badge, Spacer } from "@chakra-ui/react";
import { useBitcoinPrice } from "@/hooks/useBitcoinPrice";
import PriceChart from "./components/PriceChart";
import { useMarketPrice } from "@/hooks/useMarketPrice";
import { useEffect, useMemo, useState } from "react";
import PurchaseNowActionBar from "../market/[id]/_components/PurchaseNowActionBar";
import { useMarketStore } from "@/hooks/store/marketStore";

export default function BitcoinPage() {
    const btc = useBitcoinPrice("BTCUSDT");
    const [debouncedMarketId, setDebouncedMarketId] = useState(btc.marketId);
    const market = useMarketStore((s) => s.selectedMarket);



    const formatTime = (seconds: number) => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    };

    useEffect(() => {
        if (btc.marketId && btc.marketId !== debouncedMarketId) {
            setDebouncedMarketId(btc.marketId);
        }
    }, [btc.marketId]);

    const marketPrice = useMarketPrice(debouncedMarketId);

    const yesPercent = (marketPrice.latestYesPrice * 100).toFixed(1);
    const noPercent = (marketPrice.latestNoPrice * 100).toFixed(1);

    useEffect(() => {
        console.log("📊 BTC STATE:", btc);
    }, [btc]);

    const hasMarketId = !!debouncedMarketId;
    const isValidMarketId = btc.marketId !== "00000000-0000-0000-0000-000000000000";
    const isSafeTime = btc.lessThan40Secs === false;
    const showActionBar = isValidMarketId && hasMarketId && isSafeTime;
    return (
        <div className="w-full flex justify-center">
            <div className="w-full max-w-7xl px-6 mx-auto space-y-6">

                {/* ================================
        CONDITIONAL RENDERING:
        - If market exists → show market box
        - If no market → show Price Chart
       ================================ */}
                <div
                    style={{
                        width: "100%",
                        height: "650px",
                        position: "relative",
                        borderRadius: "12px",
                        overflow: "hidden",
                        boxShadow: "sm",
                        background: "white",
                    }}
                >
                    {/* =========================
      BACKGROUND CONTENT
     ========================= */}

                    {!market ? (
                        <PriceChart symbol="BTCUSDT" />
                    ) : (
                        <Box w="100%" h="100%" position="relative">

                            {/* optional dimmed chart background feel */}
                            <Box opacity={0.08} position="absolute" inset={0}>
                                <PriceChart symbol="BTCUSDT" />
                            </Box>

                            {/* MARKET OVERLAY */}
                            <Box
                                position="absolute"
                                inset={0}
                                display="flex"
                                alignItems="center"
                                justifyContent="center"
                                px={6}
                            >
                                <Box
                                    bg="rgba(255,255,255,0.9)"
                                    backdropFilter="blur(12px)"
                                    borderRadius="2xl"
                                    p={8}
                                    textAlign="center"
                                    shadow="lg"
                                    border="1px solid"
                                    borderColor="gray.200"
                                    minW="320px"
                                >
                                    {/* OUTCOME */}
                                    <Box mb={5}>
                                        <Text fontSize="sm" color="gray.500" textTransform="uppercase">
                                            Market Outcome
                                        </Text>

                                        <Text
                                            fontSize="3xl"
                                            fontWeight="bold"
                                            color={
                                                market.finalOutcome === 1
                                                    ? "green.500"
                                                    : market.finalOutcome === 2
                                                        ? "red.500"
                                                        : "gray.500"
                                            }
                                        >
                                            {market.finalOutcome === 1 && "YES (Above)"}
                                            {market.finalOutcome === 2 && "NO (Below)"}
                                            {market.finalOutcome === 0 && "Unspecified"}
                                            {market.finalOutcome === 3 && "Unresolved"}
                                        </Text>
                                    </Box>

                                    {/* RESOLUTION */}
                                    <Box>
                                        <Text fontSize="sm" color="gray.500" textTransform="uppercase">
                                            Resolution Criteria
                                        </Text>

                                        <Box
                                            mt={2}
                                            bg="gray.50"
                                            p={4}
                                            borderRadius="lg"
                                            fontSize="sm"
                                            fontFamily="mono"
                                            textAlign="left"
                                        >
                                            {(() => {
                                                try {
                                                    const res =
                                                        typeof market.resolutionCriteria === "string"
                                                            ? JSON.parse(market.resolutionCriteria)
                                                            : market.resolutionCriteria;

                                                    return (
                                                        <>
                                                            <Box display="flex" justifyContent="space-between"> <HStack> <Text>Condition</Text> <Text fontWeight="bold"> was the price {res?.condition}? </Text> </HStack> </Box>

                                                            <Box display="flex" justifyContent="space-between">
                                                                <Text>Price to beat</Text>
                                                                <Text fontWeight="bold">{res?.target_price}</Text>
                                                            </Box>

                                                            <Box display="flex" justifyContent="space-between">
                                                                <Text>Resolved</Text>
                                                                <Text fontWeight="bold">{res?.resolved_price}</Text>
                                                            </Box>
                                                        </>
                                                    );
                                                } catch {
                                                    return <Text color="gray.400">Invalid resolution data</Text>;
                                                }
                                            })()}
                                        </Box>
                                    </Box>
                                </Box>


                            </Box>

                            {/* ✅ BUTTON pinned INSIDE chart box */}
                            <Box
                                position="absolute"
                                bottom={4}
                                left="50%"
                                transform="translateX(-50%)"
                            >
                                <Button
                                    colorScheme="blue"
                                    size="md"
                                    borderRadius="full"
                                    px={6}
                                    onClick={() => window.location.reload()}
                                >
                                    🚀 Go to Live Market
                                </Button>
                            </Box>

                        </Box>
                    )}
                </div>
                {/* YES / NO Percentage blocks */}
                <div className="flex gap-4">
                    <div className="flex-1 bg-gradient-to-br from-green-500 to-green-700 rounded-xl p-6 text-white text-center">
                        <div className="text-sm uppercase">ABOVE</div>
                        <div className="text-4xl font-bold mt-2">{yesPercent}%</div>
                    </div>

                    <div className="flex-1 bg-gradient-to-br from-pink-500 to-pink-700 rounded-xl p-6 text-white text-center">
                        <div className="text-sm uppercase">BELOW</div>
                        <div className="text-4xl font-bold mt-2">{noPercent}%</div>
                    </div>
                </div>
                {showActionBar ? (
                    <PurchaseNowActionBar market_id={debouncedMarketId} isBTC />
                ) : null}

            </div>
        </div>
    );
}
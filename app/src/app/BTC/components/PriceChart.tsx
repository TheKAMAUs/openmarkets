"use client";

import { Chart, useChart } from "@chakra-ui/charts";
import { Box, Button, ButtonGroup, Flex, Text } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState, useRef } from "react";
import {
    CartesianGrid,
    Line,
    LineChart,
    Tooltip,
    XAxis,
    YAxis,
    ReferenceLine,
    ResponsiveContainer,
    Area,
    ComposedChart
} from "recharts";
import Image from "next/image";
import { TrendingUp, TrendingDown, Clock, Target } from "lucide-react";

import { useBitcoinPrice } from "@/hooks/useBitcoinPrice";

type Props = {
    symbol?: string; // Default "BTCUSDT"
};

const TIMELINE_FILTERS = ["1H", "6H", "24H", "7D", "ALL"] as const;
type TimelineFilter = (typeof TIMELINE_FILTERS)[number];

const PriceChart = ({ symbol = "BTCUSDT" }: Props) => {
    const [graphTimelineFilter, setGraphTimelineFilter] = useState<TimelineFilter>("1H");

    // ✅ Use Bitcoin price hook for real-time updates
    const btcData = useBitcoinPrice(symbol);

    const [priceHistory, setPriceHistory] = useState<
        { price: number; time: string; target?: number }[]
    >([]);

    const processedTimestamps = useRef<Set<number>>(new Set());

    // Initialize with current price
    useEffect(() => {
        if (btcData.currentPrice > 0) {
            const timestamp = btcData.timestamp;

            if (!processedTimestamps.current.has(timestamp)) {
                processedTimestamps.current.add(timestamp);

                setPriceHistory((prev) => {
                    // Keep last 100 points max
                    const newHistory = [
                        ...prev,
                        {
                            price: btcData.currentPrice,
                            target: btcData.targetPrice,
                            time: timestamp && !isNaN(Number(timestamp))
                                ? new Date(Number(timestamp)).toISOString()
                                : new Date().toISOString(),
                        },
                    ];
                    return newHistory.slice(-100);
                });
            }
        }
    }, [btcData.currentPrice, btcData.timestamp, btcData.targetPrice]);

    // Format price for display
    const formatPrice = (price: number) => {
        return `$${price.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
    };

    // Format percentage
    const formatPercent = (value: number) => {
        return `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
    };


    // Custom heartbeat dot at the end of the line
    const renderHeartbeatDot = (props: any) => {
        const { cx, cy, payload, index } = props;
        const isLastPoint = index === priceHistory.length - 1;

        if (!isLastPoint) return null;

        return (
            <g>
                {/* Outer pulsing ring */}
                <circle
                    cx={cx}
                    cy={cy}
                    r={8}
                    fill="none"
                    stroke={btcData.isAboveTarget ? "#16a34a" : "#ef4444"}
                    strokeWidth={2}
                    opacity={0.6}
                >
                    <animate
                        attributeName="r"
                        from="8"
                        to="16"
                        dur="1.5s"
                        repeatCount="indefinite"
                    />
                    <animate
                        attributeName="opacity"
                        from="0.6"
                        to="0"
                        dur="1.5s"
                        repeatCount="indefinite"
                    />
                </circle>

                {/* Inner pulsing ring */}
                <circle
                    cx={cx}
                    cy={cy}
                    r={4}
                    fill="none"
                    stroke={btcData.isAboveTarget ? "#16a34a" : "#ef4444"}
                    strokeWidth={2}
                    opacity={0.8}
                >
                    <animate
                        attributeName="r"
                        from="4"
                        to="10"
                        dur="1.5s"
                        repeatCount="indefinite"
                        begin="0.3s"
                    />
                    <animate
                        attributeName="opacity"
                        from="0.8"
                        to="0"
                        dur="1.5s"
                        repeatCount="indefinite"
                        begin="0.3s"
                    />
                </circle>

                {/* Center dot */}
                <circle
                    cx={cx}
                    cy={cy}
                    r={5}
                    fill={btcData.isAboveTarget ? "#16a34a" : "#ef4444"}
                    stroke="white"
                    strokeWidth={2}
                >
                    <animate
                        attributeName="r"
                        values="5;6;5"
                        dur="1s"
                        repeatCount="indefinite"
                    />
                </circle>

                {/* Price label above the dot */}
                <text
                    x={cx}
                    y={cy - 20}
                    textAnchor="middle"
                    fontSize={12}
                    fontWeight="bold"
                    fill={btcData.isAboveTarget ? "#16a34a" : "#ef4444"}
                    className="drop-shadow-sm"
                >
                    {formatPrice(btcData.currentPrice)}
                </text>
            </g>
        );
    };

    // Get gradient color based on trend
    const getGradientColor = () => {
        if (priceHistory.length < 2) return btcData.isAboveTarget ? "#16a34a" : "#ef4444";

        const firstPrice = priceHistory[0]?.price;
        const lastPrice = priceHistory[priceHistory.length - 1]?.price;
        const isUptrend = lastPrice > firstPrice;

        return isUptrend ? "#16a34a" : "#ef4444";
    };

    const lineColor = getGradientColor();

    const PRICE_WINDOW = 400;
    const HALF = PRICE_WINDOW / 2;



    const chart = useChart({
        data: priceHistory,
        series: [
            {
                name: "price",
                color: btcData.isAboveTarget ? "green.600" : "red.400",
                label: "BTC Price"
            },
        ],
    });




    return (
        <Box w="100%" display="flex" justifyContent="center">
            <Box w="100%" maxW="1200px" px={4}>

                {/* HEADER */}
                <Box mt={5} mb={6} w="100%">
                    <Flex
                        w="100%"
                        justifyContent="space-between"
                        alignItems="center"
                        wrap="wrap"
                        gap={10}
                    >

                        {/* LEFT: PRICE BLOCK */}
                        <Flex direction="column" gap={3}>
                            <Box>
                                <Text fontSize="sm" color="gray.500">
                                    Current Price
                                </Text>
                                <Text fontWeight="bold" fontSize="2xl">
                                    {formatPrice(btcData.currentPrice)}
                                </Text>
                            </Box>

                            <Box>
                                <Text fontSize="sm" color="gray.500">
                                    Price to beat
                                </Text>
                                <Text fontWeight="bold" fontSize="xl">
                                    {formatPrice(btcData.targetPrice)}
                                </Text>
                            </Box>
                        </Flex>

                        {/* CENTER: MARKET STATE */}
                        <Flex direction="column" alignItems="center" gap={3}>
                            <Box textAlign="center">
                                <Text fontSize="sm" color="gray.500">
                                    Difference
                                </Text>
                                <Text
                                    fontWeight="bold"
                                    fontSize="xl"
                                    color={btcData.priceDifference >= 0 ? "green.600" : "red.400"}
                                >
                                    {formatPercent(btcData.percentToTarget)}
                                </Text>
                            </Box>

                            <Flex alignItems="center" gap={2}>
                                {btcData.isAboveTarget ? (
                                    <TrendingUp size={18} className="text-green-600" />
                                ) : (
                                    <TrendingDown size={18} className="text-red-400" />
                                )}

                                <Text
                                    fontWeight="bold"
                                    fontSize="lg"
                                    color={btcData.isAboveTarget ? "green.600" : "red.400"}
                                >
                                    {btcData.isAboveTarget ? "ABOVE" : "BELOW"}
                                </Text>
                            </Flex>
                        </Flex>

                        {/* RIGHT: TIME + LOGO */}
                        <Flex direction="column" alignItems="flex-end" gap={3}>
                            <Box textAlign="right">
                                <Text fontSize="sm" color="gray.500">
                                    Next Target
                                </Text>
                                <Text fontWeight="bold" fontSize="xl" color="orange.500">
                                    {Math.floor(btcData.secondsRemaining / 60)}:
                                    {(btcData.secondsRemaining % 60).toString().padStart(2, "0")}
                                </Text>
                            </Box>

                            <Image
                                className="pointer-events-none select-none"
                                src="https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTJHh416u6o5_GJUMGventF-FwPg7MN-YvPlA&s"
                                alt="Logo"
                                width={135}
                                height={23}
                                style={{ opacity: 0.6, borderRadius: '8px' }}
                            />
                        </Flex>

                    </Flex>
                </Box>
                {/* Chart - With Heartbeat at the end */}
                <Box
                    width="100%"
                    height="450px"
                    position="relative"
                    bg="white"
                    borderRadius="xl"
                    p={4}
                    shadow="sm"
                    overflow="hidden"
                >
                    {/* WATERMARK BACKGROUND */}
                    <Box
                        position="absolute"
                        inset={0}
                        display="flex"
                        alignItems="center"
                        justifyContent="center"
                        opacity={0.08}
                        pointerEvents="none"
                        zIndex={0}
                    >
                        <Image
                            src="https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTJHh416u6o5_GJUMGventF-FwPg7MN-YvPlA&s"
                            alt="watermark"
                            fill
                            style={{
                                objectFit: "contain",
                            }}
                        />
                    </Box>
                    <ResponsiveContainer width="100%" height="100%">
                        <ComposedChart
                            data={priceHistory}
                            margin={{ top: 20, right: 30, left: 20, bottom: 20 }}
                        >
                            <defs>
                                <linearGradient id="lineGradient" x1="0" y1="0" x2="1" y2="0">
                                    <stop offset="0%" stopColor={lineColor} stopOpacity={0.3} />
                                    <stop offset="100%" stopColor={lineColor} stopOpacity={1} />
                                </linearGradient>
                                <linearGradient id="areaGradient" x1="0" y1="0" x2="0" y2="1">
                                    <stop offset="0%" stopColor={lineColor} stopOpacity={0.2} />
                                    <stop offset="100%" stopColor={lineColor} stopOpacity={0} />
                                </linearGradient>
                            </defs>

                            <CartesianGrid stroke="#f0f0f0" strokeDasharray="5 5" vertical={false} />

                            {/* Target Price Reference Line */}
                            {btcData.targetPrice > 0 && (
                                <ReferenceLine
                                    y={btcData.targetPrice}
                                    stroke="#f59e0b"
                                    strokeDasharray="8 4"
                                    strokeWidth={2}
                                    label={{
                                        value: `🎯 Target: ${formatPrice(btcData.targetPrice)}`,
                                        position: "insideTopRight",
                                        fill: "#f59e0b",
                                        fontSize: 12,
                                        fontWeight: "bold",
                                    }}
                                />
                            )}

                            <XAxis
                                dataKey="time"
                                axisLine={false}
                                tickLine={false}
                                tickFormatter={(value) => {
                                    const date = new Date(value);
                                    if (graphTimelineFilter === "1H" || graphTimelineFilter === "6H") {
                                        return date.toLocaleTimeString("en-US", {
                                            hour: "2-digit",
                                            minute: "2-digit",
                                        });
                                    }
                                    return date.toLocaleDateString("en-US", {
                                        month: "short",
                                        day: "numeric",
                                    });
                                }}
                                stroke="#94a3b8"
                                interval="preserveStartEnd"
                                tick={{ fontSize: 11, fill: "#64748b" }}
                            />

                            <YAxis
                                tickLine={false}
                                axisLine={false}
                                tickFormatter={(value) => formatPrice(value)}
                                orientation="right"
                                domain={[
                                    (dataMin: number) => Math.min(dataMin, btcData.currentPrice - HALF),
                                    (dataMax: number) => Math.max(dataMax, btcData.currentPrice + HALF),
                                ]}
                                stroke="#94a3b8"
                                tick={{ fontSize: 11, fill: "#64748b" }}
                                width={85}
                            />

                            <Tooltip
                                wrapperStyle={{ outline: 'none' }}
                                contentStyle={{ outline: 'none' }}
                                content={({ active, payload, label }) => {
                                    if (!active || !payload || !payload.length || !label) {
                                        return null;
                                    }

                                    const dataPoint = payload[0]?.payload;
                                    const date = new Date(label).toLocaleString("en-US", {
                                        year: "numeric",
                                        month: "short",
                                        day: "numeric",
                                        hour: "2-digit",
                                        minute: "2-digit",
                                        second: "2-digit",
                                    });

                                    return (
                                        <div className="bg-white rounded-lg shadow-lg p-4 border border-gray-200">
                                            <div className="text-sm font-semibold text-gray-600 mb-2">
                                                {date}
                                            </div>
                                            <div className="text-lg font-bold" style={{ color: lineColor }}>
                                                {formatPrice(payload[0]?.value as number)}
                                            </div>
                                            {dataPoint?.target && (
                                                <div className="text-xs text-orange-500 mt-1">
                                                    Target: {formatPrice(dataPoint.target)}
                                                </div>
                                            )}
                                            <div className="text-xs text-gray-400 mt-2">
                                                Live Price Feed
                                            </div>
                                        </div>
                                    );
                                }}
                            />

                            {/* Area under the line */}
                            <Area
                                type="monotone"
                                dataKey="price"
                                stroke="none"
                                fill="url(#areaGradient)"
                                isAnimationActive={false}
                            />

                            {/* Main Line */}
                            <Line
                                type="monotone"
                                dataKey="price"
                                stroke="url(#lineGradient)"
                                strokeWidth={3}
                                dot={false}
                                activeDot={{ r: 6, strokeWidth: 2, stroke: "white" }}
                                isAnimationActive={false}
                            />

                            {/* Heartbeat dot at the end - Custom render */}
                            <Line
                                type="monotone"
                                dataKey="price"
                                stroke="none"
                                dot={renderHeartbeatDot}
                                isAnimationActive={false}
                            />
                        </ComposedChart>
                    </ResponsiveContainer>
                </Box>
                {/* Timeline buttons */}
                {/* <Flex mt={4} alignItems="start" justifyContent="space-between">
                <ButtonGroup variant="subtle" size="sm" gap={0}>
                    {TIMELINE_FILTERS.map((filter) => (
                        <Button
                            key={filter}
                            value={filter}
                            onClick={() => setGraphTimelineFilter(filter)}
                            backgroundColor={
                                graphTimelineFilter === filter ? "gray.200" : "gray.50"
                            }
                            _hover={{ backgroundColor: "gray.100" }}
                            _active={{ backgroundColor: "gray.300" }}
                        >
                            {filter}
                        </Button>
                    ))}
                </ButtonGroup>
            </Flex> */}

                {/* CSS for blinking animation */}
                <style jsx>{`
    @keyframes blinker {
        0% { opacity: 0.3; r: 8; }
        50% { opacity: 0.8; r: 12; }
        100% { opacity: 0.3; r: 8; }
    }
    
    /* Remove focus rings from all chart elements */
    :global(.recharts-wrapper),
    :global(.recharts-wrapper:focus),
    :global(.recharts-surface),
    :global(.recharts-surface:focus),
    :global(.recharts-tooltip-wrapper),
    :global(.recharts-tooltip-wrapper:focus),
    :global(.recharts-cartesian-grid),
    :global(.recharts-reference-line),
    :global(svg:focus),
    :global(svg *) {
        outline: none !important;
        box-shadow: none !important;
    }
    
    /* Remove default focus ring from any clickable chart element */
    :global(.recharts-bar-rectangles:focus),
    :global(.recharts-sector:focus),
    :global(.recharts-layer:focus) {
        outline: none !important;
    }
`}</style>
            </Box>
        </Box>
    );
};

export default PriceChart;
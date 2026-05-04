"use client";

import { Chart, useChart } from "@chakra-ui/charts";
import { Box, Button, ButtonGroup, Flex, Text } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import { FileUp, Settings } from "lucide-react";
import Image from "next/image";
import { useEffect, useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { ChartGetters } from "@/utils/interactions/dataGetter";
import useSubscription from "@/hooks/useSubscription";

import { MarketMessage, WsParamsPayload } from "@/generated/service_types/ws_server/market_price";
import { Timeframe } from "@/generated/grpc_service_types/common";

type Props = {
  market_id: string;
};

const PriceChart = ({ market_id }: Props) => {
  const [graphTimelineFilter, setGraphTimelineFilter] =
    useState<(typeof PAST_DAYS_FILTERS)[number]>("ALL");

  const [priceData, setPriceData] = useState<
    { yes: number; no: number; time: string }[]
  >([]);
  const [labelsData, setLabelsData] = useState({
    yes: 0,
    no: 0,
  });

  // Step 2: Subscribe to WebSocket for real-time updates
  const { messages } = useSubscription<MarketMessage>(
    "/proto/proto_defs/ws_server/market_price.proto",
    "ws_market_price.MarketMessage",
    {
      payload: {
        type: "Subscribe",
        data: {
          channel: `price_update:${market_id}`,
        },
      },
    },
    false,
  );


  const { data: resp } = useQuery({
    queryKey: ["chartData", graphTimelineFilter],
    queryFn: () =>
      ChartGetters.getChartDataWithinTimeRange(
        market_id,
        fromChartArrayIdxToFilterTypeEnum(graphTimelineFilter),
      ),
  });

  useEffect(() => {
    if (resp?.priceData) {
      setPriceData(
        resp.priceData.map((item) => ({
          yes: item.yesPrice,
          no: item.noPrice,
          time: new Date(Number(item.timestamp)).toISOString(), // ✅ Convert bigint → number
        })),
      );
    }
  }, [resp?.priceData]);

  useEffect(() => {
    // set labeled data to the latest yes and no prices
    if (priceData.length > 0) {
      const latestData = priceData[priceData.length - 1];
      setLabelsData({
        yes: latestData.yes * 100, // convert to percentage
        no: latestData.no * 100, // convert to percentage
      });
    }
  }, [priceData]);

  // For chart data (accumulate all)
  useEffect(() => {
    if (!messages || messages.length === 0) return;

    setPriceData((prevData) => {
      const existingTimestamps = new Set(prevData.map(d => d.time));

      const newData = messages
        .filter(msg => "params" in msg) // Filter only valid messages
        .map((msg) => {
          const price = msg.params as WsParamsPayload;
          return {
            yes: price.yesPrice ?? 0,
            no: price.noPrice ?? 0,
            time: new Date(Number(price.timestamp) * 1000).toISOString(),
          };
        })
        .filter(newItem => !existingTimestamps.has(newItem.time));

      if (newData.length === 0) return prevData;
      return [...prevData, ...newData];
    });
  }, [messages]);


  const chart = useChart({
    data: priceData,
    series: [
      { name: "yes", color: "green.600" },
      { name: "no", color: "red.400" },
    ],
  });

  return (
    <Box mt={5}>
      {/* current yes / no price and logo */}
      <Flex mt={4} mb={6} justifyContent="space-between" alignItems="center">
        <Flex gap={4} alignItems="center">
          <Text fontWeight="bold" fontSize="sm" color="green.600">
            Yes {labelsData.yes.toFixed(2)}%
          </Text>
          <Text fontWeight="bold" fontSize="sm" color="red.400">
            No {labelsData.no.toFixed(2)}%
          </Text>
        </Flex>
        <Image
          className="pointer-events-none opacity-50 select-none"
          src="/assets/logo.svg"
          alt="Logo"
          width={135}
          height={23}
        />
      </Flex>
      <Box w="100%" h="sm">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chart.data}>

            <CartesianGrid stroke={chart.color("gray.200")} vertical={false} />

            <XAxis
              axisLine={false}
              dataKey={chart.key("time")}
              tickFormatter={(value) =>
                new Date(value).toLocaleDateString("en-US", {
                  year: "numeric",
                  month: "short",
                  day: "numeric",
                })
              }
              stroke={chart.color("border")}
              interval="preserveStartEnd"
              ticks={
                chart.data.length > 6
                  ? chart.data
                    .filter(
                      (_, i) =>
                        i % Math.ceil(chart.data.length / 5) === 0 ||
                        i === chart.data.length - 1
                    )
                    .map((d) => d.time)
                  : undefined
              }
            />

            <Tooltip
              animationDuration={100}
              cursor={{ strokeDasharray: "3 3", stroke: chart.color("gray.400") }}
              content={({ active, payload, label }) => {
                if (!active || !payload || !payload.length || label === undefined)
                  return null;

                const date = new Date(label).toLocaleString("en-US", {
                  year: "numeric",
                  month: "short",
                  day: "numeric",
                  hour: "2-digit",
                  minute: "2-digit",
                });

                return (
                  <Box bg="white" p={2} borderRadius="md" boxShadow="md">
                    <Text fontWeight="bold" mb={1}>
                      {date}
                    </Text>

                    {payload.map((entry: any, index) => (
                      <Text
                        key={entry.dataKey + index}
                        color={chart.color(entry.stroke)}
                        fontWeight="semibold"
                      >
                        {entry.name?.toUpperCase()}:{" "}
                        {(entry.value * 100).toFixed(2)}%
                      </Text>
                    ))}
                  </Box>
                );
              }}
            />

            <YAxis
              tickLine={false}
              axisLine={false}
              tickFormatter={(value) => `${Math.round(value * 100)}%`}
              orientation="right"
              domain={[0, 1]}
            />

            {chart.series.map((item, idx) => {
              const lastIndex = chart.data.length - 1;

              return (
                <Line
                  key={(item?.name ?? "") + idx}
                  dataKey={chart.key(item.name)}
                  stroke={chart.color(item.color)}
                  strokeWidth={2}
                  type="natural"
                  dot={(props) => {
                    if (props.index !== lastIndex) return null;
                    if (props.cx === undefined || props.cy === undefined) return null;

                    return (
                      <g>
                        <circle
                          cx={props.cx}
                          cy={props.cy}
                          r={3}
                          fill={chart.color(item.color)}
                          opacity={0.8}
                        />
                        <circle
                          cx={props.cx}
                          cy={props.cy}
                          r={6}
                          fill={chart.color(item.color)}
                          style={{
                            animation: "blinker 2s linear infinite",
                            opacity: 0.5,
                          }}
                        />
                        <text
                          x={props.cx}
                          y={props.cy - 14}
                          textAnchor="middle"
                          fontSize={12}
                          fontWeight="bold"
                          fill={chart.color(item.color)}
                        >
                          {item.name?.toUpperCase()}
                        </text>
                      </g>
                    );
                  }}
                />
              );
            })}
          </LineChart>
        </ResponsiveContainer>
      </Box>

      {/* timeline buttons */}
      <Flex mt={4} alignItems="start" justifyContent="space-between">
        <ButtonGroup variant="subtle" size="sm" gap={0}>
          {PAST_DAYS_FILTERS.map((filter) => (
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
      </Flex>

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
  );
};

export default PriceChart;

const PAST_DAYS_FILTERS = ["1H", "6H", "1D", "1W", "1M", "ALL"] as const;

function fromChartArrayIdxToFilterTypeEnum(
  item: (typeof PAST_DAYS_FILTERS)[number],
): Timeframe {
  switch (item) {
    case "1H":
      return Timeframe.ONE_HOUR;
    case "6H":
      return Timeframe.SIX_HOUR;
    case "1D":
      return Timeframe.ONE_DAY;
    case "1W":
      return Timeframe.ONE_WEEK;
    case "1M":
      return Timeframe.ONE_MONTH;
    case "ALL":
      return Timeframe.ALL;
    default:
      throw new Error("Invalid timeframe");
  }
}

// hooks/useMarketPrice.ts
import { MarketMessage, WsParamsPayload } from '@/generated/service_types/ws_server/market_price';
import { useState, useEffect, useRef } from 'react';
import useSubscription from './useSubscription';
import { ChartGetters } from '@/utils/interactions/dataGetter';
import { Timeframe } from '@/generated/grpc_service_types/common';


export const useMarketPrice = (market_id: string) => {
    const [marketPrice, setMarketPrice] = useState({
        marketId: market_id,
        latestYesPrice: 0,
        latestNoPrice: 0,
    });

    const [isHistoricalLoaded, setIsHistoricalLoaded] = useState(false);
    const processedTimestamps = useRef<Set<number>>(new Set());

    // console.log(`🔵 [useMarketPrice] Hook called for market: ${market_id}`);

    // Step 1: Fetch historical price data FIRST (this will show prices immediately)
    useEffect(() => {
        const fetchHistoricalPrice = async () => {
            try {
                console.log(`🔵 [useMarketPrice] Fetching historical price for market: ${market_id}`);

                const resp = await ChartGetters.getChartDataWithinTimeRange(
                    market_id,
                    fromChartArrayIdxToFilterTypeEnum("ALL")
                );

                // console.log(`🔵 [useMarketPrice] Historical response:`, resp);

                if (resp?.priceData && resp.priceData.length > 0) {
                    const latestHistorical = resp.priceData[resp.priceData.length - 1];

                    // console.log(`🔵 [useMarketPrice] ✅ Historical data loaded - YES: ${latestHistorical.yesPrice}, NO: ${latestHistorical.noPrice}`);

                    setMarketPrice({
                        marketId: market_id,
                        latestYesPrice: latestHistorical.yesPrice,
                        latestNoPrice: latestHistorical.noPrice,
                    });
                } else {
                    console.warn(`⚠️ [useMarketPrice] No historical data found, using defaults`);
                    // Set default 0.5 prices if no historical data
                    setMarketPrice({
                        marketId: market_id,
                        latestYesPrice: 0.5,
                        latestNoPrice: 0.5,
                    });
                }

                setIsHistoricalLoaded(true);
            } catch (error) {
                console.error(`❌ [useMarketPrice] Failed to fetch historical data:`, error);
                // Set default prices on error
                setMarketPrice({
                    marketId: market_id,
                    latestYesPrice: 0.5,
                    latestNoPrice: 0.5,
                });
                setIsHistoricalLoaded(true);
            }
        };

        fetchHistoricalPrice();
    }, [market_id]);

    // Step 2: Subscribe to WebSocket for real-time updates
    const { messages } = useSubscription<MarketMessage>(
        "/proto/proto_defs/ws_server/market_price.proto",
        "ws_market_price.MarketMessage", // ✅ correct
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

    // console.log(`🔵 [useMarketPrice] WebSocket messages received: ${messages?.length || 0}`);

    // Step 3: Update with WebSocket messages ONLY if we have historical data loaded
    useEffect(() => {
        if (!isHistoricalLoaded) return;
        if (!messages || messages.length === 0) return;

        const latestMessage = messages[messages.length - 1];
        console.log("📩 RAW MESSAGE:", latestMessage);

        // ---- HANDLE PRICE PAYLOAD ----
        if ("params" in latestMessage) {
            const price = latestMessage.params as WsParamsPayload;

            const timestamp = price.timestamp;
            if (timestamp && processedTimestamps.current.has(Number(timestamp))) {
                console.log("🔵 Duplicate price message skipped");
                return;
            }

            if (timestamp) {
                processedTimestamps.current.add(Number(timestamp));
            }

            setMarketPrice({
                marketId: market_id,
                latestYesPrice: price.yesPrice ?? marketPrice.latestYesPrice,
                latestNoPrice: price.noPrice ?? marketPrice.latestNoPrice,
            });


        }
    }, [messages, market_id, isHistoricalLoaded]);

    return marketPrice;
};

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



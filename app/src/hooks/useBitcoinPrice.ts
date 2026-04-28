import { useState, useEffect, useRef } from "react";
import useSubscription from "./useSubscription";
import { MarketMessage } from "@/generated/service_types/ws_server/market_price";

// ✅ Updated interface with ALL fields from server


// Define the Bitcoin price payload type with camelCase (as it comes from the WebSocket)
interface BitcoinPricePayload {
    symbol: string;
    marketId: string;           // ✅ camelCase
    currentPrice: number;       // ✅ camelCase
    targetPrice: number;        // ✅ camelCase
    targetTimestamp: number;    // ✅ camelCase
    secondsRemaining: number;   // ✅ camelCase
    isAboveTarget: boolean;     // ✅ camelCase
    timestamp: number;          // ✅ camelCase
    priceDifference: number;    // ✅ camelCase
    percentToTarget: number;    // ✅ camelCase
    lessThan40Secs: boolean;    // ✅ camelCase
}

export const useBitcoinPrice = (symbol: string = "BTCUSDT") => {
    const [bitcoinPrice, setBitcoinPrice] = useState({
        symbol: "",
        marketId: "",              // camelCase for React
        currentPrice: 0,
        targetPrice: 0,
        targetTimestamp: 0,
        secondsRemaining: 0,
        isAboveTarget: false,
        timestamp: 0,
        priceDifference: 0,
        percentToTarget: 0,
        lessThan40Secs: false,     // camelCase for React
    });

    const { messages } = useSubscription<MarketMessage>(
        "/proto/proto_defs/ws_server/market_price.proto",
        "ws_market_price.MarketMessage",
        {
            payload: {
                type: "Subscribe",
                data: {
                    channel: `bitcoin_price:${symbol}`,
                },
            },
        },
        false,
    );

    useEffect(() => {
        if (messages && messages.length > 0) {
            const latestMessage = messages[messages.length - 1];

            if ("price" in latestMessage) {
                const price = latestMessage.price as BitcoinPricePayload;
                console.log("📩 RAW MESSAGE:", latestMessage);
                if (!price) return;

                const seconds = Number(price.secondsRemaining);

                setBitcoinPrice({
                    symbol: price.symbol ?? symbol,
                    marketId: price.marketId ?? "",
                    currentPrice: price.currentPrice ?? 0,
                    targetPrice: price.targetPrice ?? 0,
                    targetTimestamp: price.targetTimestamp ?? 0,
                    secondsRemaining: seconds ?? 0,
                    isAboveTarget: price.isAboveTarget ?? false,
                    timestamp: price.timestamp ?? Date.now(),
                    priceDifference: price.priceDifference ?? 0,
                    percentToTarget: price.percentToTarget ?? 0,
                    lessThan40Secs: seconds < 40,
                });
            }
        }
    }, [messages, symbol]);

    return bitcoinPrice;
};
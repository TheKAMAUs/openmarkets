"use client";

import { Box, Button, Flex, Text } from "@chakra-ui/react";
import { useEffect, useState } from "react";

import MarketOrderForm from "./MarketOrderForm";
import LimitOrderForm from "./LimitOrderForm";
import { MarketPrice } from "@/generated/grpc_service_types/markets";
import { formatPriceString } from "@/utils";
import { StopLossOrderForm } from "./StopLossOrderForm";



type Props = {
  mode: "buy" | "sell";
  orderType: "market" | "limit" | "stop_loss";
  market_id: string;
  marketPrice: MarketPrice;
};

const TradeForm = ({ mode, orderType, market_id, marketPrice }: Props) => {
  const [stockMode, setStockMode] = useState<"yes" | "no">("yes");
  const yesPrice = formatPriceString(marketPrice.latestYesPrice);
  const noPrice = formatPriceString(marketPrice.latestNoPrice);

  // Get current price based on selected outcome
  const currentPrice = stockMode === "yes"
    ? marketPrice.latestYesPrice
    : marketPrice.latestNoPrice;

  // Calculate suggested stop price based on mode and current price
  const getSuggestedStopPrice = () => {
    if (mode === "buy") {
      return (currentPrice * 1.02).toFixed(4); // 2% above current
    } else {
      return (currentPrice * 0.98).toFixed(4); // 2% below current
    }
  };

  const [suggestedStopPrice, setSuggestedStopPrice] = useState(getSuggestedStopPrice());

  // Update suggested stop price when current price or mode changes
  useEffect(() => {
    setSuggestedStopPrice(getSuggestedStopPrice());
  }, [currentPrice, stockMode]);

  return (
    <Box>
      <Flex gap={2} width="100%" justifyContent="space-between">
        <Button
          width="1/2"
          bg={stockMode === "yes" ? "green.600" : "gray.500"}
          _hover={{ bg: "green.600" }}
          onClick={() => setStockMode("yes")}
          py={6}
          rounded="lg"
        >
          Yes
          <Text fontSize="md" fontWeight="bold" color="white">
            {yesPrice}
          </Text>
        </Button>
        <Button
          width="1/2"
          bg={stockMode === "no" ? "red.600" : "gray.500"}
          _hover={{ bg: "red.600" }}
          onClick={() => setStockMode("no")}
          py={6}
          rounded="lg"
        >
          No
          <Text fontSize="md" fontWeight="bold" color="white">
            {noPrice}
          </Text>
        </Button>
      </Flex>

      {/* Market Order Form */}
      {orderType === "market" && (
        <MarketOrderForm
          mode={mode}
          stockMode={stockMode}
          market_id={market_id}
          priceAtExecution={currentPrice}
        />
      )}

      {/* Limit Order Form */}
      {orderType === "limit" && (
        <LimitOrderForm
          mode={mode}
          stockMode={stockMode}
          market_id={market_id}
        />
      )}

      {/* Stop-Loss Order Form */}
      {orderType === "stop_loss" && (
        <StopLossOrderForm
          mode={mode}
          stockMode={stockMode}
          market_id={market_id}
          currentPrice={currentPrice}

        />
      )}
    </Box>
  );
};

export default TradeForm;
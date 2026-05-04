import { Box, VStack, Text, Input, Button, Flex } from "@chakra-ui/react";
import { useState } from "react";
import { toaster } from "@/components/ui/toaster";

interface StopLossOrderFormProps {
    mode: "buy" | "sell";
    stockMode: "yes" | "no";
    market_id: string;
    currentPrice: number;
}

export const StopLossOrderForm = ({
    mode,
    stockMode,
    market_id,
    currentPrice,
}: StopLossOrderFormProps) => {
    // Calculate suggested stop price based on current price and mode
    const getSuggestedStopPrice = () => {
        if (mode === "buy") {
            return (currentPrice * 1.02).toFixed(4); // 2% above for buy stop
        } else {
            return (currentPrice * 0.98).toFixed(4); // 2% below for sell stop
        }
    };

    const [stopPrice, setStopPrice] = useState(getSuggestedStopPrice());
    const [amount, setAmount] = useState("");
    const [isLoading, setIsLoading] = useState(false);

    // Update suggested price when currentPrice or mode changes
    const updateSuggestedPrice = () => {
        setStopPrice(getSuggestedStopPrice());
    };

    // Optional: Add a button to reset to suggested price
    const resetToSuggested = () => {
        setStopPrice(getSuggestedStopPrice());
        toaster.create({
            title: "Reset to suggested price",
            description: getSuggestedStopPrice(),
            type: "info",
            duration: 2000
        });
    };

    const handleSubmit = async () => {
        const stopPriceNum = parseFloat(stopPrice);
        const amountNum = parseFloat(amount);

        if (!stopPriceNum || stopPriceNum <= 0) {
            toaster.create({ title: "Invalid stop price", type: "error" });
            return;
        }

        if (!amountNum || amountNum <= 0) {
            toaster.create({ title: "Invalid amount", type: "error" });
            return;
        }

        if ((mode === "buy" && stopPriceNum <= currentPrice) ||
            (mode === "sell" && stopPriceNum >= currentPrice)) {
            toaster.create({
                title: `Stop price must be ${mode === "buy" ? "above" : "below"} current price`,
                description: `Current: ${currentPrice}, Stop: ${stopPriceNum}`,
                type: "error"
            });
            return;
        }

        setIsLoading(true);

        try {
            // API call here
            await new Promise(resolve => setTimeout(resolve, 1000));

            toaster.create({ title: "Stop order created!", type: "success" });
            setAmount("");
            // Don't reset stop price, keep it for next order
        } catch (error) {
            toaster.create({ title: "Failed to create order", type: "error" });
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <Box mt={4}>
            <VStack gap={4}>
                <Text fontSize="sm" color="gray.600">
                    {mode === "buy"
                        ? `🔴 Buy ${stockMode.toUpperCase()} when price rises to ${currentPrice || 'stop price'}`
                        : `🟢 Sell ${stockMode.toUpperCase()} when price falls to ${currentPrice || 'stop price'}`}
                </Text>

                <Box width="100%">
                    <Flex justify="space-between" align="center" mb={1}>
                        <Text>Stop Price</Text>
                        <Button
                            size="xs"
                            variant="ghost"
                            onClick={resetToSuggested}
                            title="Reset to suggested price"
                        >
                            Suggested: {getSuggestedStopPrice()}
                        </Button>
                    </Flex>
                    <Input
                        type="number"
                        value={stopPrice}
                        onChange={(e) => setStopPrice(e.target.value)}
                        placeholder={getSuggestedStopPrice()}
                        step="0.0001"
                    />
                    <Text fontSize="xs" color="gray.500" mt={1}>
                        Current: {currentPrice} • {mode === "buy" ? "Need >" : "Need <"} {currentPrice}
                    </Text>
                </Box>

                <Box width="100%">
                    <Text mb={1}>Amount (KES)</Text>
                    <Input
                        type="number"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                        placeholder="Enter amount"
                    />
                </Box>

                {amount && stopPrice && (
                    <Box bg="gray.100" p={2} borderRadius="md" width="100%">
                        <Text fontSize="sm">
                            {mode === "buy" ? "Buy" : "Sell"} {amount} KES {stockMode.toUpperCase()} @ {stopPrice}
                        </Text>
                        <Text fontSize="sm">≈ {(parseFloat(amount) / parseFloat(stopPrice)).toFixed(2)} shares</Text>
                    </Box>
                )}

                <Button
                    colorScheme={mode === "buy" ? "green" : "red"}
                    onClick={handleSubmit}
                    loading={isLoading}
                    width="100%"
                >
                    Create Stop Order
                </Button>
            </VStack>
        </Box>
    );
};
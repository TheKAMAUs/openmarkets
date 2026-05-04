"use client";

import * as React from "react";
import { Text as ChakraText, Box, Button, DialogCloseTrigger, Field, Fieldset, Input, Stack, VStack } from "@chakra-ui/react";

import {
    Radio,
    RadioGroup
} from "@/components/ui/radio";
import {
    DialogActionTrigger,
    DialogBody,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogRoot,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";

import { Market, MarketStatus } from "@/generated/grpc_service_types/markets";
import { MarketActions } from "@/utils/interactions/dataPosters";
import { toaster } from "./ui/toaster";
import { useMutation } from "@tanstack/react-query";
import useRevalidation from "@/hooks/useRevalidate";

interface Props {
    market: Market;
    isInitialized?: boolean;  // Pass from parent
}

export const AdminMarketButtons = ({ market, isInitialized = false }: Props) => {

    // ✅ log at the very start
    console.log("🟢 Component initialized, isInitialized:", isInitialized);


    const [initOpen, setInitOpen] = React.useState(false);
    const [finalizeOpen, setFinalizeOpen] = React.useState(false);
    const [paramsOpen, setParamsOpen] = React.useState(false);
    const [depth, setDepth] = React.useState(5);
    const [quantity, setQuantity] = React.useState(100);
    const [depthError, setDepthError] = React.useState(false);
    const [quantityError, setQuantityError] = React.useState(false);
    const [finalOutcome, setFinalOutcome] = React.useState<"yes" | "no">("yes");

    const { mutateAsync, isPending } = useMutation({
        mutationFn: MarketActions.initializeMarket,
    });

    const { mutateAsync: finalizeAsync, isPending: isFinalizePending } = useMutation({
        mutationFn: MarketActions.finalizeMarket,
    });

    const revalidate = useRevalidation();

    // Check if market is finalized
    const isFinalized = market.status === MarketStatus.SETTLED;

    const handleInitialize = async () => {
        if (!market?.id) {
            toaster.error({
                title: "Market ID is required",
            });
            return;
        }

        toaster.promise(
            mutateAsync({
                market_id: market.id,
                depth: depth,
                quantity: quantity,
            }),
            {
                loading: {
                    title: "Initializing market...",
                    description: `Creating ${depth * quantity * 4} bootstrap orders`
                },
                success: () => {
                    setParamsOpen(false);
                    revalidate(["market", market.id]);
                    revalidate(["marketOrders", market.id]);
                    return {
                        title: "Market initialized successfully!",
                        description: `Market is now live with bootstrap liquidity`
                    };
                },
                error: (error: any) => {
                    console.error("Error initializing market:", error);
                    return {
                        title: "Failed to initialize market",
                        description: error?.message || "Please try again",
                    };
                },
            }
        );
    };

    const handleFinalize = async () => {
        if (!market?.id) {
            toaster.error({
                title: "Market ID is required",
            });
            return;
        }

        if (!finalOutcome) {
            toaster.error({
                title: "Final outcome is required",
                description: "Please select YES or NO as the winning outcome",
            });
            return;
        }

        toaster.promise(
            finalizeAsync({
                market_id: market.id,
                final_outcome: finalOutcome,
            }),
            {
                loading: {
                    title: "Finalizing market...",
                    description: `Settling market with outcome: ${finalOutcome === "yes" ? "✅ YES" : "❌ NO"}`
                },
                success: () => {
                    setFinalizeOpen(false);
                    revalidate(["market", market.id]);
                    revalidate(["marketOrders", market.id]);
                    revalidate(["userData"]);
                    return {
                        title: "Market finalized successfully!",
                        description: `Market settled with outcome: ${finalOutcome}`
                    };
                },
                error: (error: any) => {
                    console.error("Error finalizing market:", error);
                    return {
                        title: "Failed to finalize market",
                        description: error?.message || "Please try again",
                    };
                },
            }
        );
    };

    // If market is finalized, show nothing
    if (isFinalized) {
        return null;
    }

    return (
        <Stack direction="row" gap="4">
            {/* Initialize Market Dialog - Only show if NOT initialized */}
            {!isInitialized && (
                <DialogRoot open={initOpen} onOpenChange={(e) => setInitOpen(e.open)} placement="center">
                    <DialogTrigger asChild>
                        <Button colorPalette="green" size="sm">Initialize Market</Button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Initialize Market</DialogTitle>
                            <DialogCloseTrigger />
                        </DialogHeader>
                        <DialogBody>
                            <ChakraText>Are you sure you want to initialize this market?</ChakraText>
                            <ChakraText fontSize="sm" color="gray.500" mt={2}>
                                This will create bootstrap liquidity and make the market tradable.
                            </ChakraText>
                        </DialogBody>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setInitOpen(false)}>
                                Cancel
                            </Button>
                            <Button
                                colorPalette="green"
                                onClick={() => {
                                    setInitOpen(false);
                                    setParamsOpen(true);
                                }}
                            >
                                Continue to Parameters
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </DialogRoot>
            )}

            {/* Parameters Dialog - Only show if NOT initialized */}
            {!isInitialized && (
                <DialogRoot open={paramsOpen} onOpenChange={(e) => setParamsOpen(e.open)} placement="center">
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Market Parameters</DialogTitle>
                            <DialogCloseTrigger />
                        </DialogHeader>
                        <DialogBody>
                            <VStack gap={6}>
                                <Field.Root required invalid={depthError}>
                                    <Field.Label>
                                        Depth <Field.RequiredIndicator />
                                    </Field.Label>
                                    <Input
                                        type="number"
                                        min={1}
                                        max={100}
                                        value={depth}
                                        onChange={(e) => {
                                            setDepth(Number(e.target.value));
                                            setDepthError(false);
                                        }}
                                        onBlur={() => setDepthError(depth < 1 || depth > 100)}
                                        placeholder="Enter depth (5)"
                                    />
                                    <Field.HelperText>
                                        Number of price levels (1-100)
                                    </Field.HelperText>
                                    <Field.ErrorText>
                                        Depth must be between 1 and 100
                                    </Field.ErrorText>
                                </Field.Root>

                                <Field.Root required invalid={quantityError}>
                                    <Field.Label>
                                        Quantity <Field.RequiredIndicator />
                                    </Field.Label>
                                    <Input
                                        type="number"
                                        min={1}
                                        max={1000}
                                        value={quantity}
                                        onChange={(e) => {
                                            setQuantity(Number(e.target.value));
                                            setQuantityError(false);
                                        }}
                                        onBlur={() => setQuantityError(quantity < 1 || quantity > 1000)}
                                        placeholder="Enter quantity (100)"
                                    />
                                    <Field.HelperText>
                                        Orders per price level (1-1000)
                                    </Field.HelperText>
                                    <Field.ErrorText>
                                        Quantity must be between 1 and 1000
                                    </Field.ErrorText>
                                </Field.Root>

                                <Box
                                    p={4}
                                    bg="blue.50"
                                    borderRadius="md"
                                    width="full"
                                    border="1px solid"
                                    borderColor="blue.200"
                                >
                                    <ChakraText fontWeight="bold" color="blue.700" mb={2}>📊 Summary</ChakraText>
                                    <ChakraText color="blue.600">
                                        Total orders: <strong>{depth * quantity * 4}</strong>
                                    </ChakraText>
                                </Box>
                            </VStack>
                        </DialogBody>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setParamsOpen(false)}>
                                Back
                            </Button>
                            <Button
                                colorPalette="green"
                                onClick={handleInitialize}
                                loading={isPending}
                                disabled={!depth || !quantity || depth < 1 || quantity < 1}
                            >
                                Initialize Market
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </DialogRoot>
            )}

            {/* Finalize Dialog - Only show if initialized AND not finalized */}
            {/* {isInitialized && !isFinalized && ( */}
            <DialogRoot open={finalizeOpen} onOpenChange={(e) => setFinalizeOpen(e.open)} placement="center">
                <DialogTrigger asChild>
                    <Button colorPalette="orange" size="sm">Finalize Market</Button>
                </DialogTrigger>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Finalize Market</DialogTitle>
                    </DialogHeader>
                    <DialogBody>
                        <RadioGroup value={finalOutcome} onValueChange={(e) => setFinalOutcome(e.value as "yes" | "no")}>
                            <Stack direction="row" gap="4">
                                <Radio value="yes">YES</Radio>
                                <Radio value="no">NO</Radio>
                            </Stack>
                        </RadioGroup>
                    </DialogBody>
                    <DialogFooter>
                        <DialogActionTrigger asChild>
                            <Button variant="outline">Cancel</Button>
                        </DialogActionTrigger>
                        <Button colorPalette="orange" loading={isFinalizePending} onClick={handleFinalize}>
                            Confirm
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </DialogRoot>
            {/* )} */}
        </Stack>
    );
};






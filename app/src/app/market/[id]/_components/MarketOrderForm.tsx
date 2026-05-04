"use client";

import { Box, Button, Flex, NumberInput, Text, VStack } from "@chakra-ui/react";
import { useCallback, useEffect, useState } from "react";
import { useMutation } from "@tanstack/react-query";

import { toaster } from "@/components/ui/toaster";
import useUserInfo from "@/hooks/useUserInfo";
import useRevalidation from "@/hooks/useRevalidate";
import { formatPriceString } from "@/utils";
import { MarketActions } from "@/utils/interactions/dataPosters";

type Props = {
  mode: "buy" | "sell";
  stockMode: "yes" | "no";
  market_id: string;
  priceAtExecution: number;
};

const MarketOrderForm = ({ mode, stockMode, market_id, priceAtExecution }: Props) => {
  const [amount, setAmount] = useState("");
  const [shares, setShares] = useState("");
  const [validationMessage, setValidationMessage] = useState<string | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [isTradeValid, setIsTradeValid] = useState(true);
  const { data: userInfo } = useUserInfo();
  const { mutateAsync, isPending } = useMutation({
    mutationFn: MarketActions.createMarketOrder,
  });
  const revalidate = useRevalidation();

  console.log('📊 Current Prices market order form:', {
    priceAtExecution,
    mode,
    stockMode
  });

  // Debounced validation function
  const validateTradeAmount = useCallback(
    debounce(async (value: string) => {
      if (!value || Number(value) <= 0) {
        setValidationMessage(null);
        setIsTradeValid(true);
        return;
      }

      const numAmount = Number(value);

      // For sell mode, convert shares to amount
      const amountToValidate = mode === "buy" ? numAmount : numAmount * priceAtExecution;

      setIsValidating(true);
      try {
        const response = await MarketActions.validateTrade({
          market_id,
          outcome: stockMode,
          amount: amountToValidate,
        });

        setIsTradeValid(response.is_valid);
        setValidationMessage(response.is_valid ? null : response.message);

        console.log('✅ Validation result:', response);
      } catch (error: any) {
        console.error('❌ Validation error:', error);
        setIsTradeValid(false);
        setValidationMessage(error?.message || "Validation failed");
      } finally {
        setIsValidating(false);
      }
    }, 500),
    [market_id, stockMode, mode, priceAtExecution]
  );

  // Trigger validation when amount changes (buy mode)
  useEffect(() => {
    if (mode === "buy") {
      validateTradeAmount(amount);
    }
  }, [amount, mode, validateTradeAmount]);

  // Trigger validation when shares changes (sell mode)
  useEffect(() => {
    if (mode === "sell") {
      validateTradeAmount(shares);
    }
  }, [shares, mode, validateTradeAmount]);

  function handleSubmit() {
    // console.log("🔵 [handleSubmit] START");
    // console.log("🔵 [handleSubmit] mode:", mode);

    // console.log("🔵 [handleSubmit] stockMode:", stockMode);
    // console.log("🔵 [handleSubmit] amount:", amount);
    // console.log("🔵 [handleSubmit] shares:", shares);
    // console.log("🔵 [handleSubmit] priceAtExecution:", priceAtExecution);
    // console.log("🔵 [handleSubmit] userInfo:", userInfo);

    // Check trade validation before proceeding
    if (!isTradeValid) {
      console.log("🔴 [handleSubmit] Trade validation failed");
      toaster.error({
        title: "Invalid Trade",
        description: validationMessage || "This trade exceeds the allowed limits",
      });
      return;
    }

    // Validate based on mode
    if (mode === "buy" && amount === "") {
      console.log("🔴 [handleSubmit] Validation failed: Amount required for BUY");
      toaster.error({
        title: "Amount is required",
        description: "Please enter the amount in KES you want to spend",
      });
      return;
    }

    if (mode === "sell" && shares === "") {
      console.log("🔴 [handleSubmit] Validation failed: Shares required for SELL");
      toaster.error({
        title: "Shares amount is required",
        description: "Please enter the number of shares you want to sell",
      });
      return;
    }

    // Calculate values based on mode
    let amountSpent: number;  // KES amount (for BUY: what they pay, for SELL: what they receive)
    let sharesAmount: number;  // Number of shares

    if (mode === "buy") {
      // User enters KES amount they want to spend
      amountSpent = Number(amount);
      sharesAmount = Number(amount) / priceAtExecution;
      console.log("🔵 [handleSubmit] BUY calculation:");
      console.log("   amountSpent (KES):", amountSpent);
      console.log("   sharesAmount:", sharesAmount);
    } else {
      // SELL mode - User enters number of shares they want to sell
      sharesAmount = Number(shares);
      amountSpent = Number(shares) * priceAtExecution;
      console.log("🔵 [handleSubmit] SELL calculation:");
      console.log("   sharesAmount:", sharesAmount);
      console.log("   amountSpent (KES):", amountSpent);
    }

    // Validate sufficient balance for buy orders
    if (mode === "buy") {
      const userBalance = userInfo?.balance || 0;
      console.log("🔵 [handleSubmit] Balance check - User balance:", userBalance);
      console.log("🔵 [handleSubmit] Balance check - Required:", amountSpent);

      if (userBalance < amountSpent) {
        console.log("🔴 [handleSubmit] Insufficient balance!");
        console.log(`   Need: ${amountSpent}, Have: ${userBalance}, Shortfall: ${amountSpent - userBalance}`);
        toaster.error({
          title: "Insufficient balance",
          description: `You need ${formatCurrency(amountSpent)} but have ${formatCurrency(userBalance)}`,
        });
        return;
      }
      console.log("✅ [handleSubmit] Balance check passed");
    }

    // Validate sufficient shares for sell orders (commented out but with log)
    // if (mode === "sell") {
    //     const userShares = userInfo?.shares || 0;
    //     console.log("🔵 [handleSubmit] Shares check - User shares:", userShares);
    //     console.log("🔵 [handleSubmit] Shares check - Required:", sharesAmount);
    //     // Uncomment when ready
    //     // if (userShares < sharesAmount) {
    //     //     console.log("🔴 [handleSubmit] Insufficient shares!");
    //     //     toaster.error({
    //     //         title: "Insufficient shares",
    //     //         description: `You need ${sharesAmount.toFixed(2)} shares but have ${userShares.toFixed(2)}`,
    //     //     });
    //     //     return;
    //     // }
    // }

    console.log("🔵 [handleSubmit] Preparing mutation payload...");
    const payload = {
      market_id,
      outcome: stockMode,
      side: mode,
      amount_spent: amountSpent,
      price_at_execution: priceAtExecution,
      ...(mode === "sell" && { shares_to_sell: Number(shares) }),
    };
    console.log("🔵 [handleSubmit] Payload:", JSON.stringify(payload, null, 2));

    console.log("🔵 [handleSubmit] Calling mutateAsync...");

    toaster.promise(
      mutateAsync(payload),
      {
        loading: {
          title: `${mode === "buy" ? "Buying" : "Selling"} ${stockMode.toUpperCase()}...`,
          description: `Market order being processed`,
        },
        success: (data) => {
          console.log("✅ [handleSubmit] Order successful!");
          console.log("✅ [handleSubmit] Response data:", data);

          setAmount("");
          setShares("");
          setValidationMessage(null);

          console.log("🔵 [handleSubmit] Revalidating queries...");
          revalidate(["marketOrders", market_id]);
          revalidate(["userData"]);
          revalidate(["userHoldings", market_id]);

          return {
            title: "Order created successfully!",
            description: `${mode === "buy" ? "Bought" : "Sold"} ${sharesAmount.toFixed(2)} ${stockMode.toUpperCase()} shares`,
          };
        },
        error: (error: any) => {
          console.error("🔴 [handleSubmit] Order failed!");
          console.error("🔴 [handleSubmit] Error:", error);
          console.error("🔴 [handleSubmit] Error message:", error?.message);
          console.error("🔴 [handleSubmit] Error response:", error?.response?.data);

          return {
            title: "Order Failed",
            description: error?.message || "Failed to create order. Please try again.",
          };
        },
      },
    );

    console.log("🔵 [handleSubmit] END - mutation in progress");
  }

  // Debounce utility function
  function debounce<T extends (...args: any[]) => any>(
    func: T,
    wait: number
  ): (...args: Parameters<T>) => void {
    let timeout: NodeJS.Timeout | null = null;
    return (...args: Parameters<T>) => {
      if (timeout) clearTimeout(timeout);
      timeout = setTimeout(() => func(...args), wait);
    };
  }

  // Calculate shares preview for buy mode
  const sharesPreview = mode === "buy" && amount ? (Number(amount) / priceAtExecution).toFixed(2) : null;

  // Calculate cost preview for sell mode
  const costPreview = mode === "sell" && shares ? (Number(shares) * priceAtExecution).toFixed(2) : null;

  return (
    <Box>
      <Flex mt={4}>
        <Box width="full">
          <Text fontSize="lg" color="gray.600" fontWeight="semibold">
            {mode === "buy" ? "Amount" : "Shares"}
          </Text>
          <Text fontSize="sm" color="gray.500" fontWeight="medium">
            {mode === "buy" ? `Bal. ${formatPriceString(userInfo?.balance || 0)}` : null}
          </Text>
        </Box>

        {mode === "buy" ? (
          <Box width="full">
            <NumberInput.Root
              formatOptions={{
                style: "currency",
                currency: "USD",
                currencyDisplay: "symbol",
                currencySign: "accounting",
              }}
            >
              <NumberInput.Input
                width="full"
                dir="rtl"
                outline="none"
                border="none"
                placeholder="$10"
                fontSize="4xl"
                fontWeight="extrabold"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </NumberInput.Root>
            {isValidating && (
              <Text fontSize="xs" color="gray.500" mt={1}>
                Validating...
              </Text>
            )}
          </Box>
        ) : (
          <Box width="full">
            <NumberInput.Root>
              <NumberInput.Input
                width="full"
                dir="rtl"
                outline="none"
                border="none"
                placeholder="10 shares"
                fontSize="4xl"
                fontWeight="extrabold"
                value={shares}
                onChange={(e) => setShares(e.target.value)}
              />
            </NumberInput.Root>
            {isValidating && (
              <Text fontSize="xs" color="gray.500" mt={1}>
                Validating...
              </Text>
            )}
          </Box>
        )}
      </Flex>

      {/* Validation message */}
      {validationMessage && !isTradeValid && (
        <Box mt={2} p={2} bg="red.50" borderRadius="md" border="1px solid" borderColor="red.200">
          <Text fontSize="sm" color="red.600">
            ⚠️ {validationMessage}
          </Text>
        </Box>
      )}

      {/* Preview for buy mode - shows how many shares you'll get */}
      {mode === "buy" && sharesPreview && Number(amount) > 0 && (
        <Flex mt={2} justifyContent="space-between" alignItems="center">
          <Text fontSize="sm" color="gray.500">
            You will receive:
          </Text>
          <Text fontSize="sm" fontWeight="semibold" color="blue.600">
            {sharesPreview} {stockMode.toUpperCase()} shares
          </Text>
        </Flex>
      )}

      {/* Preview for sell mode - shows how much you'll get after fees */}
      {mode === "sell" && costPreview && Number(shares) > 0 && (
        <VStack mt={2} gap={2} align="stretch">
          <Flex justifyContent="space-between" alignItems="center">
            <Text fontSize="sm" color="gray.500">
              Total value:
            </Text>
            <Text fontSize="sm" fontWeight="semibold" color="gray.700">
              {formatPriceString(Number(costPreview))}
            </Text>
          </Flex>

          <Flex justifyContent="space-between" alignItems="center">
            <Text fontSize="sm" color="gray.500">
              Trading fee (1%):
            </Text>
            <Text fontSize="sm" color="red.500">
              -{formatPriceString(Number(costPreview) * 0.01)}
            </Text>
          </Flex>

          <Flex justifyContent="space-between" alignItems="center">
            <Text fontSize="sm" color="gray.500">
              Platform fee (5% on after trading fee):
            </Text>
            <Text fontSize="sm" color="red.500">
              -{formatPriceString(Number(costPreview) * 0.99 * 0.05)}
            </Text>
          </Flex>

          <Flex justifyContent="space-between" alignItems="center">
            <Text fontSize="sm" fontWeight="bold" color="gray.700">
              You will receive:
            </Text>
            <Text fontSize="md" fontWeight="bold" color="green.600">
              {formatPriceString(Number(costPreview) * 0.99 * 0.95)}
            </Text>
          </Flex>
        </VStack>
      )}

      {/* Display current price */}
      <Flex mt={2} justifyContent="space-between" alignItems="center">
        <Text fontSize="sm" color="gray.500">
          Current {stockMode.toUpperCase()} Price:
        </Text>
        <Text fontSize="sm" fontWeight="semibold" color={stockMode === "yes" ? "green.600" : "red.600"}>
          {formatPriceString(priceAtExecution)}
        </Text>
      </Flex>

      {/* Predefined values - different for buy and sell */}
      <Flex mt={3} justifyContent="end" alignItems="center">
        <Flex gap={2} alignItems="center">
          {mode === "buy"
            ? PREDEFINED_AMOUNTS.map((amountValue) => (
              <Button
                key={amountValue}
                variant="outline"
                fontSize="xs"
                rounded="full"
                bg="transparent"
                border="1px solid"
                borderColor="gray.300"
                paddingX={5}
                size="xs"
                onClick={() =>
                  setAmount((prev) => (Number(prev) + amountValue).toString())
                }
              >
                ${amountValue}
              </Button>
            ))
            : PREDEFINED_SHARES.map((shareValue) => (
              <Button
                key={shareValue}
                variant="outline"
                fontSize="xs"
                rounded="full"
                bg="transparent"
                border="1px solid"
                borderColor="gray.300"
                paddingX={5}
                size="xs"
                onClick={() =>
                  setShares((prev) => (Number(prev) + shareValue).toString())
                }
              >
                {shareValue} shares
              </Button>
            ))
          }
        </Flex>
      </Flex>

      <Button
        width="full"
        mt={4}
        bg="blue.600/90"
        _hover={{ bg: "blue.600" }}
        onClick={handleSubmit}
        loading={isPending}
        disabled={!isTradeValid}
      >
        {mode === "buy" ? "Buy" : "Sell"} {stockMode.toUpperCase()} Now
      </Button>
    </Box>
  );
};

export default MarketOrderForm;

const PREDEFINED_AMOUNTS = [1, 20, 100];
const PREDEFINED_SHARES = [10, 50, 100];

function formatCurrency(amountSpent: number) {
  throw new Error("Function not implemented.");
}






// type Props = {
//   mode: "buy" | "sell";
//   stockMode: "yes" | "no";
//   market_id: string;
//   priceAtExecution: number;
// };

// const MarketOrderForm = ({ mode, stockMode, market_id, priceAtExecution }: Props) => {
//   const [amount, setAmount] = useState("");
//   const [shares, setShares] = useState("");
//   const { data: userInfo } = useUserInfo();
//   const { mutateAsync, isPending } = useMutation({
//     mutationFn: MarketActions.createMarketOrder,
//   });
//   const revalidate = useRevalidation();

//   console.log('📊 Current Prices market order form:', {
//     priceAtExecution,
//     mode,
//     stockMode
//   });



//   function handleSubmit() {
//     console.log("🔵 [handleSubmit] START");
//     console.log("🔵 [handleSubmit] mode:", mode);

//     console.log("🔵 [handleSubmit] stockMode:", stockMode);
//     console.log("🔵 [handleSubmit] amount:", amount);
//     console.log("🔵 [handleSubmit] shares:", shares);
//     console.log("🔵 [handleSubmit] priceAtExecution:", priceAtExecution);
//     console.log("🔵 [handleSubmit] userInfo:", userInfo);

//     // Validate based on mode
//     if (mode === "buy" && amount === "") {
//       console.log("🔴 [handleSubmit] Validation failed: Amount required for BUY");
//       toaster.error({
//         title: "Amount is required",
//         description: "Please enter the amount in KES you want to spend",
//       });
//       return;
//     }

//     if (mode === "sell" && shares === "") {
//       console.log("🔴 [handleSubmit] Validation failed: Shares required for SELL");
//       toaster.error({
//         title: "Shares amount is required",
//         description: "Please enter the number of shares you want to sell",
//       });
//       return;
//     }

//     // Calculate values based on mode
//     let amountSpent: number;  // KES amount (for BUY: what they pay, for SELL: what they receive)
//     let sharesAmount: number;  // Number of shares

//     if (mode === "buy") {
//       // User enters KES amount they want to spend
//       amountSpent = Number(amount);
//       sharesAmount = Number(amount) / priceAtExecution;
//       console.log("🔵 [handleSubmit] BUY calculation:");
//       console.log("   amountSpent (KES):", amountSpent);
//       console.log("   sharesAmount:", sharesAmount);
//     } else {
//       // SELL mode - User enters number of shares they want to sell
//       sharesAmount = Number(shares);
//       amountSpent = Number(shares) * priceAtExecution;
//       console.log("🔵 [handleSubmit] SELL calculation:");
//       console.log("   sharesAmount:", sharesAmount);
//       console.log("   amountSpent (KES):", amountSpent);
//     }

//     // Validate sufficient balance for buy orders
//     if (mode === "buy") {
//       const userBalance = userInfo?.balance || 0;
//       console.log("🔵 [handleSubmit] Balance check - User balance:", userBalance);
//       console.log("🔵 [handleSubmit] Balance check - Required:", amountSpent);

//       if (userBalance < amountSpent) {
//         console.log("🔴 [handleSubmit] Insufficient balance!");
//         console.log(`   Need: ${amountSpent}, Have: ${userBalance}, Shortfall: ${amountSpent - userBalance}`);
//         toaster.error({
//           title: "Insufficient balance",
//           description: `You need ${formatCurrency(amountSpent)} but have ${formatCurrency(userBalance)}`,
//         });
//         return;
//       }
//       console.log("✅ [handleSubmit] Balance check passed");
//     }

//     // Validate sufficient shares for sell orders (commented out but with log)
//     // if (mode === "sell") {
//     //     const userShares = userInfo?.shares || 0;
//     //     console.log("🔵 [handleSubmit] Shares check - User shares:", userShares);
//     //     console.log("🔵 [handleSubmit] Shares check - Required:", sharesAmount);
//     //     // Uncomment when ready
//     //     // if (userShares < sharesAmount) {
//     //     //     console.log("🔴 [handleSubmit] Insufficient shares!");
//     //     //     toaster.error({
//     //     //         title: "Insufficient shares",
//     //     //         description: `You need ${sharesAmount.toFixed(2)} shares but have ${userShares.toFixed(2)}`,
//     //     //     });
//     //     //     return;
//     //     // }
//     // }

//     console.log("🔵 [handleSubmit] Preparing mutation payload...");
//     const payload = {
//       market_id,
//       outcome: stockMode,
//       side: mode,
//       amount_spent: amountSpent,
//       price_at_execution: priceAtExecution,
//       ...(mode === "sell" && { shares_to_sell: Number(shares) }),
//     };
//     console.log("🔵 [handleSubmit] Payload:", JSON.stringify(payload, null, 2));

//     console.log("🔵 [handleSubmit] Calling mutateAsync...");

//     toaster.promise(
//       mutateAsync(payload),
//       {
//         loading: {
//           title: `${mode === "buy" ? "Buying" : "Selling"} ${stockMode.toUpperCase()}...`,
//           description: `Market order being processed`,
//         },
//         success: (data) => {
//           console.log("✅ [handleSubmit] Order successful!");
//           console.log("✅ [handleSubmit] Response data:", data);

//           setAmount("");
//           setShares("");

//           console.log("🔵 [handleSubmit] Revalidating queries...");
//           revalidate(["marketOrders", market_id]);
//           revalidate(["userData"]);
//           revalidate(["userHoldings", market_id]);

//           return {
//             title: "Order created successfully!",
//             description: `${mode === "buy" ? "Bought" : "Sold"} ${sharesAmount.toFixed(2)} ${stockMode.toUpperCase()} shares`,
//           };
//         },
//         error: (error: any) => {
//           console.error("🔴 [handleSubmit] Order failed!");
//           console.error("🔴 [handleSubmit] Error:", error);
//           console.error("🔴 [handleSubmit] Error message:", error?.message);
//           console.error("🔴 [handleSubmit] Error response:", error?.response?.data);

//           return {
//             title: "Order Failed",
//             description: error?.message || "Failed to create order. Please try again.",
//           };
//         },
//       },
//     );

//     console.log("🔵 [handleSubmit] END - mutation in progress");
//   }
//   // Calculate shares preview for buy mode
//   const sharesPreview = mode === "buy" && amount ? (Number(amount) / priceAtExecution).toFixed(2) : null;

//   // Calculate cost preview for sell mode
//   const costPreview = mode === "sell" && shares ? (Number(shares) * priceAtExecution).toFixed(2) : null;

//   return (
//     <Box>
//       <Flex mt={4}>
//         <Box width="full">
//           <Text fontSize="lg" color="gray.600" fontWeight="semibold">
//             {mode === "buy" ? "Amount" : "Shares"}
//           </Text>
//           <Text fontSize="sm" color="gray.500" fontWeight="medium">
//             {mode === "buy" ? `Bal. ${formatPriceString(userInfo?.balance || 0)}` : null}
//           </Text>
//         </Box>

//         {mode === "buy" ? (
//           <NumberInput.Root
//             formatOptions={{
//               style: "currency",
//               currency: "USD",
//               currencyDisplay: "symbol",
//               currencySign: "accounting",
//             }}
//           >
//             <NumberInput.Input
//               width="full"
//               dir="rtl"
//               outline="none"
//               border="none"
//               placeholder="$10"
//               fontSize="4xl"
//               fontWeight="extrabold"
//               value={amount}
//               onChange={(e) => setAmount(e.target.value)}
//             />
//           </NumberInput.Root>
//         ) : (
//           <NumberInput.Root>
//             <NumberInput.Input
//               width="full"
//               dir="rtl"
//               outline="none"
//               border="none"
//               placeholder="10 shares"
//               fontSize="4xl"
//               fontWeight="extrabold"
//               value={shares}
//               onChange={(e) => setShares(e.target.value)}
//             />
//           </NumberInput.Root>
//         )}
//       </Flex>

//       {/* Preview for buy mode - shows how many shares you'll get */}
//       {mode === "buy" && sharesPreview && Number(amount) > 0 && (
//         <Flex mt={2} justifyContent="space-between" alignItems="center">
//           <Text fontSize="sm" color="gray.500">
//             You will receive:
//           </Text>
//           <Text fontSize="sm" fontWeight="semibold" color="blue.600">
//             {sharesPreview} {stockMode.toUpperCase()} shares
//           </Text>
//         </Flex>
//       )}

//       {/* Preview for sell mode - shows how much you'll get after fees */}
//       {mode === "sell" && costPreview && Number(shares) > 0 && (
//         <VStack mt={2} gap={2} align="stretch">
//           <Flex justifyContent="space-between" alignItems="center">
//             <Text fontSize="sm" color="gray.500">
//               Total value:
//             </Text>
//             <Text fontSize="sm" fontWeight="semibold" color="gray.700">
//               {formatPriceString(Number(costPreview))}
//             </Text>
//           </Flex>

//           <Flex justifyContent="space-between" alignItems="center">
//             <Text fontSize="sm" color="gray.500">
//               Trading fee (1%):
//             </Text>
//             <Text fontSize="sm" color="red.500">
//               -{formatPriceString(Number(costPreview) * 0.01)}
//             </Text>
//           </Flex>

//           <Flex justifyContent="space-between" alignItems="center">
//             <Text fontSize="sm" color="gray.500">
//               Platform fee (5% on after trading fee):
//             </Text>
//             <Text fontSize="sm" color="red.500">
//               -{formatPriceString(Number(costPreview) * 0.99 * 0.05)}
//             </Text>
//           </Flex>



//           <Flex justifyContent="space-between" alignItems="center">
//             <Text fontSize="sm" fontWeight="bold" color="gray.700">
//               You will receive:
//             </Text>
//             <Text fontSize="md" fontWeight="bold" color="green.600">
//               {formatPriceString(Number(costPreview) * 0.99 * 0.95)}
//             </Text>
//           </Flex>
//         </VStack>
//       )}

//       {/* Display current price */}
//       <Flex mt={2} justifyContent="space-between" alignItems="center">
//         <Text fontSize="sm" color="gray.500">
//           Current {stockMode.toUpperCase()} Price:
//         </Text>
//         <Text fontSize="sm" fontWeight="semibold" color={stockMode === "yes" ? "green.600" : "red.600"}>
//           {formatPriceString(priceAtExecution)}
//         </Text>
//       </Flex>

//       {/* Predefined values - different for buy and sell */}
//       <Flex mt={3} justifyContent="end" alignItems="center">
//         <Flex gap={2} alignItems="center">
//           {mode === "buy"
//             ? PREDEFINED_AMOUNTS.map((amountValue) => (
//               <Button
//                 key={amountValue}
//                 variant="outline"
//                 fontSize="xs"
//                 rounded="full"
//                 bg="transparent"
//                 border="1px solid"
//                 borderColor="gray.300"
//                 paddingX={5}
//                 size="xs"
//                 onClick={() =>
//                   setAmount((prev) => (Number(prev) + amountValue).toString())
//                 }
//               >
//                 ${amountValue}
//               </Button>
//             ))
//             : PREDEFINED_SHARES.map((shareValue) => (
//               <Button
//                 key={shareValue}
//                 variant="outline"
//                 fontSize="xs"
//                 rounded="full"
//                 bg="transparent"
//                 border="1px solid"
//                 borderColor="gray.300"
//                 paddingX={5}
//                 size="xs"
//                 onClick={() =>
//                   setShares((prev) => (Number(prev) + shareValue).toString())
//                 }
//               >
//                 {shareValue} shares
//               </Button>
//             ))
//           }
//         </Flex>
//       </Flex>

//       <Button
//         width="full"
//         mt={4}
//         bg="blue.600/90"
//         _hover={{ bg: "blue.600" }}
//         onClick={handleSubmit}
//         loading={isPending}
//       >
//         {mode === "buy" ? "Buy" : "Sell"} {stockMode.toUpperCase()} Now
//       </Button>
//     </Box>
//   );
// };

// export default MarketOrderForm;

// const PREDEFINED_AMOUNTS = [1, 20, 100];
// const PREDEFINED_SHARES = [10, 50, 100];

// function formatCurrency(amountSpent: number) {
//   throw new Error("Function not implemented.");
// }
//  this code is perfectly working dont cahnge a bit only add calling validate trade method as the user types the amount
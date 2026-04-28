"use client";

import React, { useEffect, useRef, useState } from "react";
import {
  Box,
  Button,
  Flex,
  Tabs,
  useDisclosure,
  Portal, Text,
  Select,
  CloseButton,
  createListCollection,
} from "@chakra-ui/react";

import TradeForm from "./TradeForm";
import { MarketPrice } from "@/generated/grpc_service_types/markets";
import { useMarketPrice } from "@/hooks/useMarketPrice";
import { useBitcoinPrice } from "@/hooks/useBitcoinPrice";

// type Props = {
//   market_id: string;
//   marketPrice: MarketPrice;
// };

// const PurchaseNowActionBar = ({ market_id, marketPrice }: Props) => {
//   const { open: isOpen, onToggle } = useDisclosure();
//   const [orderType, setOrderType] = useState<"market" | "limit" | "stop_loss">("market");
//   const [activeTab, setActiveTab] = useState<"buy" | "sell">("buy");

//   const ctnRef = useRef<HTMLDivElement>(null);

//   useEffect(() => {
//     const handleClickOutside = (event: MouseEvent) => {
//       if (ctnRef.current && !ctnRef.current.contains(event.target as Node)) {
//         onToggle();
//       }
//     };

//     document.addEventListener("click", handleClickOutside);
//     return () => {
//       document.removeEventListener("click", handleClickOutside);
//     };
//   }, []);

//   // Determine which tabs to show based on order type
//   const getAvailableTabs = (): { value: "buy" | "sell"; label: string }[] => {
//     switch (orderType) {
//       case "limit":
//         return [{ value: "buy", label: "Buy" }]; // Limit orders only allow BUY
//       case "stop_loss":
//         return [{ value: "sell", label: "Sell" }]; // Stop loss only allows SELL
//       case "market":
//       default:
//         return [
//           { value: "buy", label: "Buy" },
//           { value: "sell", label: "Sell" },
//         ]; // Market orders allow both
//     }
//   };

//   const availableTabs = getAvailableTabs();

//   // Auto-switch tab if current tab is not available
//   useEffect(() => {
//     if (!availableTabs.find(tab => tab.value === activeTab)) {
//       setActiveTab(availableTabs[0]?.value || "buy");
//     }
//   }, [orderType, availableTabs, activeTab]);

//   return (
//     <Box
//       position="fixed"
//       left={0}
//       right={0}
//       bottom={5}
//       zIndex={10}
//       width={isOpen ? "400px" : "140px"}
//       minHeight="80px"
//       mx="auto"
//       overflow="hidden"
//       transition="all 0.2s ease-in-out"
//     >
//       {!isOpen ? (
//         <Button
//           onClick={onToggle}
//           width="100%"
//           size="lg"
//           bg="blue.subtle/50"
//           backdropBlur="md"
//           backdropFilter="blur(10px)"
//           variant="outline"
//           rounded="full"
//         >
//           Trade Now
//         </Button>
//       ) : (
//         <Box
//           bg="gray.subtle/50"
//           backdropBlur="md"
//           backdropFilter="blur(10px)"
//           boxShadow="0 -2px 8px rgba(0,0,0,0.08)"
//           px={6}
//           py={4}
//           borderRadius="xl"
//           minHeight="250px"
//           _hover={{ boxShadow: "0 -4px 12px rgba(0,0,0,0.1)" }}
//           ref={ctnRef}
//         >
//           <Tabs.Root
//             value={activeTab}
//             onValueChange={(e) => setActiveTab(e.value as "buy" | "sell")}
//           >
//             <Tabs.List
//               justifyContent={"space-between"}
//               display="flex"
//               alignItems="center"
//               gap={2}
//             >
//               <Flex>
//                 {availableTabs.map((tab) => (
//                   <Tabs.Trigger key={tab.value} value={tab.value}>
//                     {tab.label}
//                   </Tabs.Trigger>
//                 ))}
//               </Flex>
//               <Flex gap={2}>
//                 <Select.Root
//                   collection={orderTypes}
//                   size="sm"
//                   width="100px"
//                   value={[orderType]}
//                   onValueChange={(v) => setOrderType(v.value[0] as typeof orderType)}
//                 >
//                   <Select.HiddenSelect />
//                   <Select.Control>
//                     <Select.Trigger border={"none"}>
//                       <Select.ValueText placeholder="Type" />
//                     </Select.Trigger>
//                     <Select.IndicatorGroup>
//                       <Select.Indicator />
//                     </Select.IndicatorGroup>
//                   </Select.Control>
//                   <Portal>
//                     <Select.Positioner>
//                       <Select.Content bg="gray.50">
//                         {orderTypes.items.map((orderType) => (
//                           <Select.Item item={orderType} key={orderType.value}>
//                             {orderType.label}
//                             <Select.ItemIndicator />
//                           </Select.Item>
//                         ))}
//                       </Select.Content>
//                     </Select.Positioner>
//                   </Portal>
//                 </Select.Root>
//                 <CloseButton onClick={onToggle} />
//               </Flex>
//             </Tabs.List>

//             {availableTabs.map((tab) => (
//               <Tabs.Content key={tab.value} value={tab.value}>
//                 <TradeForm
//                   mode={tab.value}
//                   orderType={orderType}
//                   market_id={market_id}
//                   marketPrice={marketPrice}
//                 />
//               </Tabs.Content>
//             ))}
//           </Tabs.Root>
//         </Box>
//       )}
//     </Box>
//   );
// };

// export default PurchaseNowActionBar;

// const orderTypes = createListCollection({
//   items: [
//     { label: "Market", value: "market" },
//     { label: "Limit", value: "limit" },
//     { label: "Stop Loss", value: "stop_loss" },
//   ],
// });









type Props = {
  market_id: string;
  isBTC?: boolean; // optional flag
};
const PurchaseNowActionBar = ({ market_id, isBTC = false }: Props) => {
  // console.log("🔵 [PurchaseNowActionBar] Component rendering for market:", market_id);

  const { open: isOpen, onToggle } = useDisclosure();
  const [orderType, setOrderType] = useState<"market" | "limit" | "stop_loss">("market");
  const [activeTab, setActiveTab] = useState<"buy" | "sell">("buy");
  const orderTypeOptions = isBTC ? btcOrderTypes : orderTypes;
  // ONLY use the real-time price hook - no props
  const marketPrice = useMarketPrice(market_id);

  // console.log("🔵 [PurchaseNowActionBar] marketPrice from hook:", marketPrice);
  // console.log("🔵 [PurchaseNowActionBar] YES price:", marketPrice?.latestYesPrice);
  // console.log("🔵 [PurchaseNowActionBar] NO price:", marketPrice?.latestNoPrice);

  const ctnRef = useRef<HTMLDivElement>(null);

  // Force BTC → market only
  useEffect(() => {
    if (isBTC && orderType !== "market") {
      setOrderType("market");
    }
  }, [isBTC, orderType]);



  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (ctnRef.current && !ctnRef.current.contains(event.target as Node)) {
        onToggle();
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => {
      document.removeEventListener("click", handleClickOutside);
    };
  }, []);

  // Determine which tabs to show based on order type
  const getAvailableTabs = (): { value: "buy" | "sell"; label: string }[] => {
    switch (orderType) {
      case "limit":
        return [{ value: "buy", label: "Buy" }];
      case "stop_loss":
        return [{ value: "sell", label: "Sell" }];
      case "market":
      default:
        return [
          { value: "buy", label: "Buy" },
          { value: "sell", label: "Sell" },
        ];
    }
  };

  const availableTabs = getAvailableTabs();

  // Auto-switch tab if current tab is not available
  useEffect(() => {
    if (!availableTabs.find(tab => tab.value === activeTab)) {
      setActiveTab(availableTabs[0]?.value || "buy");
    }
  }, [orderType, availableTabs]);

  // Test subscription status
  useEffect(() => {
    const interval = setInterval(() => {
      console.log("📊 [TEST] Price check - YES:", marketPrice?.latestYesPrice, "NO:", marketPrice?.latestNoPrice);
    }, 5000);
    return () => clearInterval(interval);
  }, [marketPrice]);

  return (
    <Box
      position="fixed"
      left={0}
      right={0}
      bottom={5}
      zIndex={10}
      width={isOpen ? "400px" : "140px"}
      minHeight="80px"
      mx="auto"
      overflow="hidden"
      transition="all 0.2s ease-in-out"
    >
      {!isOpen ? (
        <Button
          onClick={onToggle}
          width="100%"
          size="lg"
          bg="blue.subtle/50"
          backdropBlur="md"
          backdropFilter="blur(10px)"
          variant="outline"
          rounded="full"
        >
          Trade Now
        </Button>
      ) : (
        <Box
          bg="gray.subtle/50"
          backdropBlur="md"
          backdropFilter="blur(10px)"
          boxShadow="0 -2px 8px rgba(0,0,0,0.08)"
          px={6}
          py={4}
          borderRadius="xl"
          minHeight="250px"
          _hover={{ boxShadow: "0 -4px 12px rgba(0,0,0,0.1)" }}
          ref={ctnRef}
        >
          {/* Display connection status */}
          <Box mb={2} p={2} bg={marketPrice?.latestYesPrice ? "green.100" : "yellow.100"} borderRadius="md">
            <Text fontSize="xs">
              {marketPrice?.latestYesPrice ? "✅ Live Prices Connected" : "⏳ Loading prices..."}
            </Text>
            <Text fontSize="xs">
              YES: {marketPrice?.latestYesPrice || "N/A"} | NO: {marketPrice?.latestNoPrice || "N/A"}
            </Text>
          </Box>

          <Tabs.Root
            value={activeTab}
            onValueChange={(e) => setActiveTab(e.value as "buy" | "sell")}
          >
            <Tabs.List
              justifyContent={"space-between"}
              display="flex"
              alignItems="center"
              gap={2}
            >
              <Flex>
                {availableTabs.map((tab) => (
                  <Tabs.Trigger key={tab.value} value={tab.value}>
                    {tab.label}
                  </Tabs.Trigger>
                ))}
              </Flex>
              <Flex gap={2}>
                <Select.Root
                  collection={orderTypeOptions}
                  size="sm"
                  width="100px"
                  value={[orderType]}
                  onValueChange={(v) =>
                    setOrderType(v.value[0] as typeof orderType)
                  }
                >
                  <Select.Control>
                    <Select.Trigger border="none">
                      <Select.ValueText placeholder="Type" />
                    </Select.Trigger>
                  </Select.Control>
                </Select.Root>
                <CloseButton onClick={onToggle} />
              </Flex>
            </Tabs.List>


            {availableTabs.map((tab) => (
              <Tabs.Content key={tab.value} value={tab.value}>
                <TradeForm
                  mode={tab.value}
                  orderType={orderType}
                  market_id={market_id}
                  marketPrice={marketPrice}
                />
              </Tabs.Content>
            ))}
          </Tabs.Root>
        </Box>
      )}
    </Box>
  );
};

export default PurchaseNowActionBar;

// ---- STATIC ORDER TYPES ----

export const orderTypes = createListCollection({
  items: [
    { label: "Market", value: "market" },
    { label: "Limit", value: "limit" },
    { label: "Stop Loss", value: "stop_loss" },
  ],
});

export const btcOrderTypes = createListCollection({
  items: [
    { label: "Market", value: "market" }, // BTC ONLY
  ],
});










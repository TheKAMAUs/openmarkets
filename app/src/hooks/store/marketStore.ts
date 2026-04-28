import { Market } from "@/generated/grpc_service_types/markets";
import { create } from "zustand";


type MarketStore = {
    selectedMarket: Market | null;
    setMarket: (market: Market | null) => void;
};
export const useMarketStore = create<MarketStore>((set) => ({
    selectedMarket: null,
    setMarket: (m) => set({ selectedMarket: m }),
}));
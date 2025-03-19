"use client";

import { createContext, useContext, useState } from "react";
import { ReactNode } from "react";

export type SideBarProfile =
  | "profile"
  | "dashboard"
  | "analytics"
  | "products"
  | "transactions"
  | "invoices"
  | "billing"
  | "chat"
  | "tickets"
  | "faq"
  | "settings"
  | "help"
  | "orders"
  | "calender"
  | "wishlists"
  | "nfts"
  | "messages";

interface SideBarContextType {
  activeComponent: SideBarProfile;
  setActiveComponent: (component: SideBarProfile) => void;
}

const SideBarContext = createContext<SideBarContextType | undefined>(undefined);

export const SideBarProvider = ({ children }: { children: ReactNode }) => {
  const [activeComponent, setActiveComponent] =
    useState<SideBarProfile>("profile");

  return (
    <SideBarContext.Provider value={{ activeComponent, setActiveComponent }}>
      {children}
    </SideBarContext.Provider>
  );
};

export const useSideBarProfile = (): SideBarContextType => {
  const context = useContext(SideBarContext);
  if (!context) {
    throw new Error("useSideBar must be used within a SideBarProvider");
  }
  return context;
};

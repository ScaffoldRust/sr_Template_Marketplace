"use client";

import React from "react";
import SellerSidebar from "./components/sidebar/ProfileSidebar";
import StoreOverview from "./components/profile-card/StoreOverview";

import { useSideBarProfile } from "./components/context/sidebarContext";

const Page = () => {
  const { activeComponent } = useSideBarProfile();

  function renderComponent() {
    switch (activeComponent) {
      case "profile":
        return <StoreOverview />;
        break;
      case "billing":
        return "billing";
        break;
      case "orders":
        return "orders";
        break;
      case "calender":
        return "calenders";
        break;
      default:
        return <StoreOverview />;
    }
  }

  return (
    <div className="min-h-screen flex flex-col md:flex-row">
      <SellerSidebar />
      <section className="flex-1 p-4">{renderComponent()}</section>
    </div>
  );
};

export default Page;

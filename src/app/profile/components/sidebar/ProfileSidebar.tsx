import React, { useState } from "react";
import SidebarNavItem from "./SidebarNavItem";

const SellerSidebar = () => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative">
      <SidebarNavItem
        isOpen={isOpen}
        toggleSidebar={() => setIsOpen(!isOpen)}
      />
    </div>
  );
};

export default SellerSidebar;

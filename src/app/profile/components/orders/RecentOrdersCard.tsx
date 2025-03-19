import Image from "next/image";
import React from "react";
import activePro from "../../../../../public/activePro.svg";
import OrderListItem from "./OrderItem";

function RecentOrdersCard() {
  return (
    <section className="border border-[#E4E4E7] rounded-lg p-7">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl text-[#09090B] font-semibold">Recent Orders</h1>
        <Image src={activePro} alt="icon" className="text-[#71717A]" />
      </div>

      <OrderListItem />
    </section>
  );
}

export default RecentOrdersCard;

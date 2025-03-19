import React from "react";
import { Delivery } from "./DeliveryCalendarCard";

interface DeliveryItem {
  order: Delivery;
}

const DeliveryItem: React.FC<DeliveryItem> = ({ order }) => {
  return (
    <div key={order.id} className="flex justify-between items-center">
      <div className="py-4">
        <h1 className="text-[#09090B] font-medium text-base">
          {order.product}
        </h1>
        <p className="text-[#71717A] text-sm">{order.date}</p>
      </div>
      <span>{order.icon}</span>
    </div>
  );
};

export default DeliveryItem;

import React from "react";
import { Order } from "./OrderItem";

const getStatusBadge = (status: Order["status"]) => {
  switch (status) {
    case "Delivered":
      return "text-[#22C55E]";
    case "Processing":
      return "text-[#EAB308]";
    case "In Transit":
      return "text-[#3B82F6]";
  }
};

interface StatusBadge {
  order: Order;
}

const OrderStatusBadge: React.FC<StatusBadge> = ({ order }) => {
  return (
    <div key={order.id} className="py-2.5 flex justify-between items-center">
      <div>
        <p className="font-medium text-[#09090B]">{order.product}</p>
        <p className="text-[#71717A] text-sm">{order.date}</p>
      </div>
      <span
        className={`px-3 font-medium py-1 text-sm rounded-lg ${getStatusBadge(
          order.status
        )}`}
      >
        {order.status}
      </span>
      <span>{order.track}</span>
    </div>
  );
};

export default OrderStatusBadge;

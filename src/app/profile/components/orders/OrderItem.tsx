import React from "react";
import OrderStatusBadge from "./StatusBadge";

export interface Order {
  id: number;
  product: string;
  date: string;
  track: string;
  status: "In Transit" | "Processing" | "Delivered";
}

const orders: Order[] = [
  {
    id: 8832,
    product: "Premium Hoodie",
    date: "March 05, 2024",
    track: "Track Order",
    status: "In Transit",
  },
  {
    id: 8831,
    product: "Urban Sneakers  ",
    date: "March 08, 2024 ",
    track: "Track Order",
    status: "Delivered",
  },
  {
    id: 8830,
    product: "Graphic T-Shirt",
    date: "March 12, 2024",
    track: "Track Order",
    status: "Processing",
  },
];

const OrderListItem = () => {
  return (
    <div className="p-4 rounded-lg border-[#E4E4E7]">
      {orders.map((order) => (
        <OrderStatusBadge key={order.id} order={order} />
      ))}
    </div>
  );
};

export default OrderListItem;

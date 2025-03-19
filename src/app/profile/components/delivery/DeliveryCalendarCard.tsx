import React from "react";
import DeliveryItem from "./DeliveryItem";
import { Calendar, Clock } from "lucide-react";

export interface Delivery {
  id: number;
  product: string;
  date: string;
  icon: React.ReactNode;
}

const deliverys: Delivery[] = [
  {
    id: 8832,
    product: "Package Arriving",
    date: "March 15, 2024",
    icon: <Clock color="#71717A" size={20} />,
  },
  {
    id: 8831,
    product: "Expected Delivery",
    date: "March 18, 2024 ",
    icon: <Clock color="#71717A" size={20} />,
  },
];

const DeliveryCalendarCard = () => {
  return (
    <div className="border border-[#E4E4E7] rounded-lg p-7">
      <div className="flex justify-between items-center text-[#09090B]">
        <p className="text-[#09090B] text-2xl font-semibold">
          Delivery Calendar
        </p>
        <span>
          <Calendar color="#71717A" size={20} />
        </span>
      </div>
      {deliverys.map((order) => (
        <DeliveryItem key={order.id} order={order} />
      ))}
    </div>
  );
};

export default DeliveryCalendarCard;

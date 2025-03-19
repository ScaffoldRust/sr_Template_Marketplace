import React from "react";
import { ArrowLeft } from "lucide-react";
import MetricCard from "./UserProfileCard";
import DeliveryCalendarCard from "../delivery/DeliveryCalendarCard";
import RecentOrdersCard from "../orders/RecentOrdersCard";
import ShoppingActivityCard from "../activity/ShoppingActivityCard";
import Link from "next/link";

const StoreOverview = () => {
  return (
    <div className="w-full mr-6 md:mr-12">
      <div className="min-h-screen mx-auto px-5 py-4">
        <div className=" max-w-6xl space-x-4 flex items-center mx-auto">
          <Link href={"/"} className="cursor-pointer">
            <ArrowLeft size={14} />
          </Link>
          <h2 className="text-2xl font-bold">My Profile</h2>
          <p className=" text-sm">Active · March, 2024</p>
        </div>
        <div className="max-w-6xl mx-auto grid mt-10 gap-6 sm:grid-cols-2 grid-cols-1">
          <MetricCard />
          <ShoppingActivityCard />
          <RecentOrdersCard />
          <DeliveryCalendarCard />
        </div>
      </div>
    </div>
  );
};

export default StoreOverview;

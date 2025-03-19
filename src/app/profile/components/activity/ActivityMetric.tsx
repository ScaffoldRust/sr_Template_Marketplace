import Image from "next/image";
import React from "react";
import activePro from "../../../../../public/activePro.svg";
import no_love from "../../../../../public/no-love.svg";
import { Star } from "lucide-react";

const ActivityMetric = () => {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Shopping Activity</h1>

      <div>
        <div className="flex justify-between mt-4 items-center">
          <span className="flex space-x-3.5">
            <Image width={20} src={activePro} alt="icon" />
            <span>
              <h3>Totals Orders</h3>
              <p className="text-[#71717A] text-sm">Currently listed</p>
            </span>
          </span>
          <p className="text-2xl font-bold">47</p>
        </div>

        <div className="flex justify-between mt-4 items-center">
          <span className="flex space-x-3.5 justify-center items-center">
            <div>
              <Star size={20} />
            </div>
            <span>
              <h3>Reviews Given</h3>
              <p className="text-[#71717A] text-sm">Customer feedback</p>
            </span>
          </span>
          <p className="text-2xl font-bold">32</p>
        </div>

        <div className="flex justify-between mt-4 items-center">
          <span className="flex space-x-3.5">
            <Image width={20} src={no_love} alt="icon" />
            <span>
              <h3>Wishlist Items</h3>
              <p className="text-[#71717A] text-sm">Saved for later</p>
            </span>
          </span>
          <p className="text-2xl font-bold">15</p>
        </div>
      </div>
    </div>
  );
};

export default ActivityMetric;

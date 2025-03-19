import { Mail } from "lucide-react";
import React from "react";
import ProfileAvatar from "./ProfileAvatar";

const MetricCard = () => {
  return (
    <section className=" text-center flex justify-center items-center flex-col w-full border border-[#E4E4E7] rounded-lg p-7">
      <ProfileAvatar />

      <span className="my-4">
        <h1 className="text-2xl font-semibold">Matias Aguilar</h1>
        <p className="text-[#A855F7] text-sm">Premium Member</p>
      </span>

      <span className=" border border-[#E4E4E7] p-2 rounded">
        <Mail size={16} />
      </span>
    </section>
  );
};
export default MetricCard;

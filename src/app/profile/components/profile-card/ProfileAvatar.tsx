import { User } from "lucide-react";
import React, { useState } from "react";
import Image from "next/image";

const ProfileAvatar = () => {
  const [avatar, setAvatar] = useState<string | null>(null);

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];

    if (file) {
      const reader = new FileReader();
      reader.onload = () => {
        setAvatar(reader.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  return (
    <div
      className={` w-fit relative rounded-full ${
        avatar ? "p-0" : "p-5"
      } bg-[#F4F4F5] `}
    >
      {avatar ? (
        <Image
          className="w-24 h-24 rounded-full object-cover"
          src={avatar}
          alt="avatar"
          width={96}
          height={96}
        />
      ) : (
        <User size={64} />
      )}
      <input
        type="file"
        accept="image/*"
        className="absolute inset-0 opacity-0 cursor-pointer"
        onChange={handleFileChange}
      />
    </div>
  );
};

export default ProfileAvatar;

import { SidebarSec } from "../seller/components/context/sidebarContext";
import { SideBarProfile } from "../profile/components/context/sidebarContext";

export interface MenuItem {
  icon: React.ReactNode;
  label: string;
  id: SidebarSec | SideBarProfile;
}

export interface MenuSection {
  title: string;
  items: MenuItem[];
}

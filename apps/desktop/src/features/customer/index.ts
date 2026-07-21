import { Users } from "@desk/ui/icons";

export const customerFeature = {
  id: "customer",
  path: "/features/customer",
  navItem: {
    id: "customer",
    path: "/features/customer",
    labelKey: "nav.customer",
    icon: Users,
  },
};

export { CustomerPage } from "./customer-page";
export { useCustomers } from "./use-customers";

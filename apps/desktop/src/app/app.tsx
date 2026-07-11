import { RouterProvider } from "react-router";
import { appRouter } from "../route";
import "./globals.css";

export function App() {
  return <RouterProvider router={appRouter} />;
}

import { Outlet } from "react-router";
import MainNavigation from "../navigation/MainNavigation";

export default function AppLayout() {
  return (
    <div className="mx-auto min-h-screen w-full max-w-app px-5 pt-6 pb-8 max-[360px]:px-4">
      <header className="flex justify-center">
        <MainNavigation />
      </header>
      <main className="mt-8">
        <Outlet />
      </main>
    </div>
  );
}

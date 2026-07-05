import Sidebar from "@/components/Sidebar";
import BottomNav from "@/components/BottomNav";
import { DashboardProvider } from "@/components/DashboardContext";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <DashboardProvider>
      <div className="flex min-h-screen bg-[#F8FAFC]">
        <Sidebar />
        <main className="flex-1 pb-16 md:pb-0">
          <div className="p-4 md:p-6">{children}</div>
        </main>
        <BottomNav />
      </div>
    </DashboardProvider>
  );
}

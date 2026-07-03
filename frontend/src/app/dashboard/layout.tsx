import Sidebar from "@/components/Sidebar";
import BottomNav from "@/components/BottomNav";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-screen bg-black">
      <Sidebar />
      <main className="flex-1 pb-16 md:pb-0">
        <div className="p-4 md:p-6">{children}</div>
      </main>
      <BottomNav />
    </div>
  );
}

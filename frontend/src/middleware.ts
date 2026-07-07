import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

// Role-based route access
const ROLE_ROUTES: Record<string, string[]> = {
  owner: [
    "/dashboard/admins",
    "/dashboard/override",
    "/dashboard/settings",
    "/dashboard/pricing",
    "/dashboard/reports",
    "/dashboard/audit-log",
  ],
  admin: [
    "/dashboard/schedule",
    "/dashboard/attendance",
    "/dashboard/billing",
    "/dashboard/students",
    "/dashboard/payments",
    "/dashboard/holidays",
    "/dashboard/studio",
    "/dashboard/teachers",
    "/dashboard/registrations",
    "/dashboard/assign-session",
  ],
  teacher: [
    "/dashboard/my-schedule",
    "/dashboard/attendance/submit",
    "/dashboard/feedback",
  ],
  guardian: [
    "/dashboard/my-schedule",
    "/dashboard/my-students",
    "/dashboard/my-payments",
    "/dashboard/ratings",
  ],
};

// All roles can access these
const COMMON_ROUTES = [
  "/dashboard",
  "/dashboard/notifications",
  "/dashboard/profile",
];

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Public routes — no auth required
  const publicRoutes = [
    "/",
    "/login",
    "/register-course",
    "/forgot-password",
    "/reset-password",
    "/instruments",
    "/help",
    "/api/",
  ];

  const isPublic = publicRoutes.some(
    (route) => pathname === route || pathname.startsWith(route)
  );

  if (isPublic) {
    return NextResponse.next();
  }

  // Check for auth token
  const token =
    request.cookies.get("access_token")?.value ||
    request.headers.get("authorization")?.replace("Bearer ", "");

  if (!token) {
    const loginUrl = new URL("/login", request.url);
    loginUrl.searchParams.set("redirect", pathname);
    return NextResponse.redirect(loginUrl);
  }

  // Decode JWT payload to get role (no verification — that's backend's job)
  try {
    const payload = JSON.parse(atob(token.split(".")[1]));
    const role = payload.role;

    if (!role) {
      return NextResponse.next();
    }

    // Check role-based access
    const allowedRoutes = [
      ...COMMON_ROUTES,
      ...(ROLE_ROUTES[role] || []),
    ];

    const hasAccess = allowedRoutes.some(
      (route) => pathname === route || pathname.startsWith(route + "/")
    );

    if (!hasAccess) {
      // Redirect to dashboard home
      return NextResponse.redirect(new URL("/dashboard", request.url));
    }
  } catch {
    // Invalid token — let backend handle
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    "/dashboard/:path*",
    "/api/v1/:path*",
  ],
};

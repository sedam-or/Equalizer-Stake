import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

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

  // Check for auth token in cookies or header
  const token =
    request.cookies.get("access_token")?.value ||
    request.headers.get("authorization")?.replace("Bearer ", "");

  if (!token) {
    // Redirect to login if no token
    const loginUrl = new URL("/login", request.url);
    loginUrl.searchParams.set("redirect", pathname);
    return NextResponse.redirect(loginUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    "/dashboard/:path*",
    "/api/v1/:path*",
  ],
};

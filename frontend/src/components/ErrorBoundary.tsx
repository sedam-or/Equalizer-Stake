"use client";
/**
 * Error Boundary component
 * T-144: Error Boundary + Error Pages
 */

import React from "react";

interface ErrorBoundaryProps {
  children: React.ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex min-h-[50vh] flex-col items-center justify-center px-6 text-center">
          <span className="mb-4 text-6xl">⚠️</span>
          <h2 className="font-heading text-xl font-bold text-gray-900">Terjadi Kesalahan</h2>
          <p className="mt-2 text-gray-500">Silakan muat ulang halaman atau hubungi admin.</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-6 rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]"
          >
            Muat Ulang
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

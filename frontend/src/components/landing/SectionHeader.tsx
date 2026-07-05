/**
 * Shared Section Header — DRY component
 */
import { SECTION_HEADER_CLASS, SECTION_SUBTITLE_CLASS } from "@/lib/landing-constants";

interface SectionHeaderProps {
  title: string;
  subtitle?: string;
}

export default function SectionHeader({ title, subtitle }: SectionHeaderProps) {
  return (
    <div className="text-center">
      <h2 className={SECTION_HEADER_CLASS}>{title}</h2>
      {subtitle && <p className={SECTION_SUBTITLE_CLASS}>{subtitle}</p>}
    </div>
  );
}

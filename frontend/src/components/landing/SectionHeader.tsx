interface SectionHeaderProps {
  title: string;
  subtitle?: string;
}

export default function SectionHeader({ title, subtitle }: SectionHeaderProps) {
  return (
    <div className="text-center">
      <h2 className="text-balance font-heading text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
        {title}
      </h2>
      {subtitle && <p className="mt-3 text-base text-gray-500 sm:text-lg">{subtitle}</p>}
    </div>
  );
}

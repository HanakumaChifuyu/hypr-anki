interface PagePlaceholderProps {
  title: string;
  description: string;
}

export default function PagePlaceholder({ title, description }: PagePlaceholderProps) {
  return (
    <section className="py-6">
      <h1 className="mb-2.5 text-[22px] leading-normal font-semibold">{title}</h1>
      <p className="text-sm leading-[1.6] text-muted">{description}</p>
    </section>
  );
}

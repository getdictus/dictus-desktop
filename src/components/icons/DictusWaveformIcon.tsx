const DictusWaveformIcon = ({
  width,
  height,
  className,
}: {
  width?: number | string;
  height?: number | string;
  className?: string;
}) => (
  <svg
    width={width || 24}
    height={height || 24}
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
    className={className}
  >
    {/* Bar 1 — left, short */}
    <rect x="4" y="9" width="4" height="6" rx="2" opacity="0.5" />
    {/* Bar 2 — center, tall */}
    <rect x="10" y="5" width="4" height="14" rx="2" />
    {/* Bar 3 — right, medium */}
    <rect x="16" y="7" width="4" height="10" rx="2" opacity="0.7" />
  </svg>
);

export default DictusWaveformIcon;

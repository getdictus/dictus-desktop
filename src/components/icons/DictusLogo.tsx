const DictusLogo = ({
  width,
  className,
}: {
  width?: number | string;
  className?: string;
}) => (
  <svg
    width={width || 200}
    viewBox="0 0 200 80"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    className={className}
  >
    <defs>
      <linearGradient
        id="dictus-bar-gradient"
        x1="35.5"
        y1="19"
        x2="44.5"
        y2="61"
        gradientUnits="userSpaceOnUse"
      >
        <stop offset="0%" stopColor="#6BA3FF" />
        <stop offset="100%" stopColor="#2563EB" />
      </linearGradient>
    </defs>
    {/* Bar 1 — left, short */}
    <rect
      x="19"
      y="31"
      width="9"
      height="18"
      rx="4.5"
      className="logo-stroke"
      fill="currentColor"
      opacity="0.45"
    />
    {/* Bar 2 — center, tall */}
    <rect
      x="35.5"
      y="19"
      width="9"
      height="42"
      rx="4.5"
      fill="url(#dictus-bar-gradient)"
    />
    {/* Bar 3 — right, medium */}
    <rect
      x="52"
      y="26"
      width="9"
      height="27"
      rx="4.5"
      className="logo-stroke"
      fill="currentColor"
      opacity="0.65"
    />
    {/* Dictus wordmark */}
    <text
      x="85"
      y="55"
      fontFamily="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
      fontWeight="600"
      fontSize="28"
      fill="currentColor"
    >
      Dictus
    </text>
  </svg>
);

export default DictusLogo;

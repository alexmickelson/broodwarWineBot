import React from 'react';

export const ExpandableSection: React.FC<{
  title: string;
  children: React.ReactNode;
  defaultExpanded?: boolean;
}> = ({ title, children, defaultExpanded = true }) => {
  const [isExpanded, setIsExpanded] = React.useState(defaultExpanded);
  const [maxHeight, setMaxHeight] = React.useState<string>('none');
  const contentRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    if (contentRef.current) {
      if (isExpanded) {
        setMaxHeight(`${contentRef.current.scrollHeight}px`);
      } else {
        setMaxHeight('0px');
      }
    }
  }, [isExpanded, children]);

  return (
    <div className="my-5 border border-void-700 hover:border-emerald-500 rounded-lg bg-linear-to-br from-abyss-200 to-abyss-300 shadow-lg hover:shadow-success transition-all duration-300">
      <h2 
        className="text-lavender-500 py-3 px-4 cursor-pointer flex items-center gap-3 border-b border-void-700 hover:bg-void-800/50 transition-colors duration-200 rounded-t-lg group"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <span 
          className="transition-all duration-300 text-slate-400 group-hover:text-emerald-500 font-bold text-lg"
          style={{ 
            transform: isExpanded ? 'rotate(0deg)' : 'rotate(-90deg)',
            textShadow: 'none'
          }}
        >
          ▼
        </span>
        <span className="font-semibold text-lg text-lavender-300 group-hover:text-emerald-300 transition-colors duration-200">{title}</span>
      </h2>
      <div
        ref={contentRef}
        className="overflow-hidden transition-all duration-500 ease-in-out"
        style={{
          maxHeight: maxHeight,
          opacity: isExpanded ? 1 : 0,
        }}
      >
        <div className="p-4 border-t border-gray-800">
          {children}
        </div>
      </div>
    </div>
  );
};

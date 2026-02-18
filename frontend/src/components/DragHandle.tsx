import { useCallback, useRef } from "react";

interface Props {
  direction: "horizontal" | "vertical";
  onRatioChange: (ratio: number) => void;
}

export default function DragHandle({ direction, onRatioChange }: Props) {
  const handleRef = useRef<HTMLDivElement>(null);
  const callbackRef = useRef(onRatioChange);
  callbackRef.current = onRatioChange;

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      const parent = handleRef.current?.parentElement;
      if (!parent) return;
      const rect = parent.getBoundingClientRect();

      const onMouseMove = (ev: MouseEvent) => {
        const ratio =
          direction === "horizontal"
            ? (ev.clientY - rect.top) / rect.height
            : (ev.clientX - rect.left) / rect.width;
        callbackRef.current(Math.max(0.1, Math.min(0.9, ratio)));
      };

      const onMouseUp = () => {
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };

      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
      document.body.style.cursor =
        direction === "horizontal" ? "row-resize" : "col-resize";
      document.body.style.userSelect = "none";
    },
    [direction]
  );

  return (
    <div
      ref={handleRef}
      className={`drag-handle drag-handle-${direction}`}
      onMouseDown={onMouseDown}
    />
  );
}

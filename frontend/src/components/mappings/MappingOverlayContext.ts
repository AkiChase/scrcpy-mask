import {
  createContext,
  useContext,
  useEffect,
  useState,
  type MouseEvent,
} from "react";

export type MappingOverlayContextValue = {
  showAllGuides: boolean;
};

export const MappingOverlayContext =
  createContext<MappingOverlayContextValue>({
    showAllGuides: false,
  });

export function useMappingGuideVisible(localVisible: boolean) {
  const { showAllGuides } = useContext(MappingOverlayContext);
  return showAllGuides || localVisible;
}

export function useMappingGuideState(active: boolean) {
  const [isHovered, setIsHovered] = useState(false);
  const [isPointerDown, setIsPointerDown] = useState(false);

  useEffect(() => {
    if (!isPointerDown) return;

    const handleMouseUp = () => setIsPointerDown(false);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isPointerDown]);

  return {
    visible: useMappingGuideVisible(active || isHovered || isPointerDown),
    interactionProps: {
      onMouseEnter: () => setIsHovered(true),
      onMouseLeave: () => setIsHovered(false),
    },
    startPointerDown: (event: MouseEvent) => {
      if (event.button === 0) {
        setIsPointerDown(true);
      }
    },
  };
}

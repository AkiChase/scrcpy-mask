import { createContext, useContext } from "react";

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

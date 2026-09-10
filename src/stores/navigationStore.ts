import { create } from "zustand";
import type { SidebarSection } from "@/components/Sidebar";

interface NavigationStore {
  section: SidebarSection;
  setSection: (section: SidebarSection) => void;
}

/**
 * Which top-level page is showing.
 *
 * This lives in a store rather than in `App` state because pages registered in
 * `SECTIONS_CONFIG` are rendered without props, and some of them need to send
 * the user somewhere else — the file transcription workspace links straight
 * into History once a transcript has been saved.
 */
export const useNavigationStore = create<NavigationStore>()((set) => ({
  section: "general",
  setSection: (section) => set({ section }),
}));

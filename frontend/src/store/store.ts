import { configureStore } from "@reduxjs/toolkit";
import { useDispatch, useSelector } from "react-redux";
import localConfig from "./localConfig";
import other from "./other";
import type { MessageInstance } from "antd/es/message/interface";

export const store = configureStore({
  reducer: {
    localConfig,
    other,
  },
});

export const staticStore: {
  messageApi: MessageInstance | null;
} = {
  messageApi: null,
};

// Infer the `RootState`,  `AppDispatch`, and `AppStore` types from the store itself
export type RootState = ReturnType<typeof store.getState>;
// Inferred type: {posts: PostsState, comments: CommentsState, users: UsersState}
export type AppDispatch = typeof store.dispatch;
export type AppStore = typeof store;

export const useAppDispatch = useDispatch.withTypes<AppDispatch>();
export const useAppSelector = useSelector.withTypes<RootState>();

import {createSlice} from '@reduxjs/toolkit'

export const internetIdentitySlice = createSlice({
  name: 'ii',
  initialState: {
    isLoggedIn: false,
  },
  reducers: {
    setLoggedIn: (state) => {
      state.isLoggedIn = true;
    },
    setLoggedOut: (state) => {
      state.isLoggedIn = false;
    },
  },
})

// Action creators are generated for each case reducer function
export const {setLoggedIn, setLoggedOut} = internetIdentitySlice.actions;


export default internetIdentitySlice.reducer

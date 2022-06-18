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
  },
})

// Action creators are generated for each case reducer function
export const {setLoggedIn} = internetIdentitySlice.actions;


export default internetIdentitySlice.reducer

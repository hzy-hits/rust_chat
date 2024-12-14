import { invoke } from '@tauri-apps/api/core';

const URL_BASE = 'http://localhost:6688/api';
const SSE_URL = 'http://localhost:6687/events';

let config = null;
try {
  if (invoke) {
    config = await invoke('get_config');
  }
} catch (error) {
  console.warn('failed to get config: fallback');
}

const getUrlBase = () => {
  if (config && config.server.chat) {
    return config.server.chat;
  }
  return URL_BASE;
}

const getSseBase = () => {
  if (config && config.server.notification) {
    return config.server.notification;
  }
  return SSE_URL;
}
const initSSE = (store) => {
    let sse_base = getSseBase();
    let url = `${sse_base}?token=${store.state.token}`;
    const sse = new EventSource(url);
  
    // new message notification
    sse.addEventListener("NewMessage", (e) => {
      let data = JSON.parse(e.data);
      console.log('message:', e.data);
      delete data.event;
      store.commit('addMessage', { channelId: data.chatId, message: data });
    });
  
    // when a new chat is created
    sse.addEventListener("NewChat", (e) => {
      let data = JSON.parse(e.data);
      console.log('NewChat event:', data);
      
      store.commit('addChannel', data);

    });
  
    // when someone is added to a chat
    sse.addEventListener("AddToChat", (e) => {
      let data = JSON.parse(e.data);
      console.log('AddToChat event:', data);


      const channelIndex = store.state.channels.findIndex(c => c.id === data.id);
      if (channelIndex !== -1) {
        // update the channel in store.state.channels
        store.state.channels.splice(channelIndex, 1, data);
        localStorage.setItem('channels', JSON.stringify(store.state.channels));
      } else {
        // if the channel does not exist in store.state.channels, add it
        store.commit('addChannel', data);
      }
    });
  
    sse.addEventListener("RemoveFromChat", (e) => {
      let data = JSON.parse(e.data);
      console.log('RemoveFromChat event:', data);
      // Same as AddToChat, but remove the user from the channel
      const channelIndex = store.state.channels.findIndex(c => c.id === data.id);
      if (channelIndex !== -1) {
        store.state.channels.splice(channelIndex, 1, data);
        localStorage.setItem('channels', JSON.stringify(store.state.channels));
      }
    });
  
    sse.addEventListener("ChatNameUpdated", (e) => {
      let data = JSON.parse(e.data);
      console.log('ChatNameUpdated event:', data);
      // update the channel name in store.state.channels
      const channelIndex = store.state.channels.findIndex(c => c.id === data.id);
      if (channelIndex !== -1) {
        store.state.channels[channelIndex].name = data.name;
        localStorage.setItem('channels', JSON.stringify(store.state.channels));
      }
    });
  
    sse.onerror = (error) => {
      console.error('EventSource failed:', error);
      sse.close();
    };
  
    return sse;
  }

export {
  getUrlBase,
  initSSE,
};

export function formatMessageDate(timestamp) {
  const date = new Date(timestamp);
  const now = new Date();
  const diffDays = Math.floor((now - date) / (1000 * 60 * 60 * 24));
  const timeString = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

  if (diffDays === 0) {
    return timeString;
  } else if (diffDays < 30) {
    return `${timeString}, ${diffDays} ${diffDays === 1 ? 'day' : 'days'} ago`;
  } else {
    return `${timeString}, ${date.toLocaleDateString([], { month: 'short', day: 'numeric', year: 'numeric' })}`;
  }
}

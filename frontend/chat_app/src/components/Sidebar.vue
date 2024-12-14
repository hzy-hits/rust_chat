<template>
  <div class="w-64 bg-gray-800 text-white flex flex-col h-screen p-4 text-sm">
    <!-- 顶部工作区与下拉菜单区域 -->
    <div class="relative mb-6">
      <div class="font-bold text-base truncate cursor-pointer flex items-center" @click="toggleDropdown">
        <span>{{ workspaceName }}</span>
        <svg class="w-4 h-4 ml-1 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.1l3.71-3.87a.75.75 0 011.08 1.04l-4.25 4.43a.75.75 0 01-1.08 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
        </svg>
      </div>
      <div
        v-if="dropdownVisible"
        class="absolute top-8 left-0 w-48 bg-gray-800 border border-gray-700 rounded-md shadow-lg z-10"
      >
        <ul class="py-1">
          <li @click="logout" class="px-4 py-2 hover:bg-gray-700 cursor-pointer">Logout</li>
          <!-- 可添加更多选项 -->
        </ul>
      </div>
    </div>

    <!-- Channels 区域 -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-xs uppercase text-gray-400">Channels</h2>
        <button
          @click="showChatModal(true)"
          class="text-gray-400 text-xl hover:text-white"
        >
          +
        </button>
      </div>
      <ul>
        <li
          v-for="channel in channels"
          :key="channel.id"
          @click="selectChannel(channel.id)"
          :class="[
            'px-2 py-1 rounded cursor-pointer hover:bg-gray-700',
            { 'bg-blue-600': channel.id === activeChannelId }
          ]"
        >
          # {{ channel.name }}
        </li>
      </ul>
    </div>

    <!-- Direct Messages 区域 -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-xs uppercase text-gray-400">Direct Messages</h2>
        <button
          @click="showChatModal(false)"
          class="text-gray-400 text-xl hover:text-white"
        >
          +
        </button>
      </div>
      <ul>
        <li
          v-for="channel in singleChannels"
          :key="channel.id"
          @click="selectChannel(channel.id)"
          :class="[
            'flex items-center px-2 py-1 rounded cursor-pointer hover:bg-gray-700',
            { 'bg-blue-600': channel.id === activeChannelId }
          ]"
        >
          <img
            :src="`https://ui-avatars.com/api/?name=${encodeURIComponent(channel.recipient.username)}`"
            class="w-6 h-6 rounded-full mr-2"
            alt="Avatar"
          />
          {{ channel.recipient.username }}
        </li>
      </ul>
    </div>

    <!-- 创建聊天模态框 -->
    <CreateChatModal
      v-if="chatModalVisible"
      :visible="chatModalVisible"
      :is-group="isGroupChat"
      @update:visible="chatModalVisible = $event"
    />
  </div>
</template>

<script>
import CreateChatModal from './CreateChatModal.vue';

export default {
  components: {
    CreateChatModal
  },
  data() {
    return {
      dropdownVisible: false,
      chatModalVisible: false,
      isGroupChat: false,
    };
  },
  computed: {
    workspaceName() {
      return this.$store.getters.getWorkspace.name || 'No Workspace';
    },
    channels() {
      return this.$store.getters.getChannels;
    },
    singleChannels() {
      return this.$store.getters.getSingChannels;
    },
    activeChannelId() {
      const channel = this.$store.state.activeChannel;
      return channel ? channel.id : null;
    },
  },
  methods: {
    toggleDropdown() {
      this.dropdownVisible = !this.dropdownVisible;
    },
    logout() {
      this.$store.dispatch('logout');
      this.$router.push('/login');
    },
    handleOutsideClick(event) {
      if (!this.$el.contains(event.target)) {
        this.dropdownVisible = false;
      }
    },
    selectChannel(channelId) {
      this.$store.dispatch('setActiveChannel', channelId);
    },
    showChatModal(isGroup) {
      this.isGroupChat = isGroup;
      this.chatModalVisible = true;
    },
  },
  mounted() {
    document.addEventListener('click', this.handleOutsideClick);
  },
  beforeDestroy() {
    document.removeEventListener('click', this.handleOutsideClick);
  },
};
</script>

<style scoped>
/* 可根据需要添加或修改CSS */
</style>

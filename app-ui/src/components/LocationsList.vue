<template>
  <div v-if="live?.length > 0 || base?.length > 0">
    <div class="table-container">
      <table class="table is-fullwidth is-hoverable">
        <tr>
          <th>ID</th>
          <th>Name</th>
          <th>Address</th>
          <th>Link</th>
        </tr>
        <tr v-for="location in live ?? base" :key="location.id">
          <td>{{ location.id }}</td>
          <td>{{ location.name }}</td>
          <td>{{ location.address ?? '--' }}</td>
          <td>
            <router-link
              v-bind:to="`/locations/${location.id}`"
              class="button is-link"
              >View Location</router-link
            >
          </td>
        </tr>
      </table>
    </div>
  </div>
  <div v-else>
    <p>No locations added to the inventory tracking system.</p>
  </div>
</template>
<script>
import { useClientHandle } from '@urql/vue';

export default {
  async setup() {
    const handle = useClientHandle();

    // get the initial locations
    let result = await handle
      .useQuery({
        query: `
      {
        locations {
          id,
          name,
          address,
        }
      }
      `,
      })
      .executeQuery();
    result = JSON.parse(JSON.stringify(result.data.value.locations));

    // update the locations live
    const handleSubscription = (locations = result, response) => {
      const event = response.locationSubscription;
      if (event.modification === 'CREATE') {
        // naive insert to sorted location
        let i = 0;
        for (; i < locations.length; i++) {
          if (locations[i].name >= event.data.name) {
            break;
          }
        }
        locations.splice(i, 0, event.data);
        return locations;
      } else if (event.modification === 'UPDATE') {
        locations = locations.filter(
          (location) => location.id !== event.data.id
        );
        // naive insert to sorted location
        let i = 0;
        for (; i < locations.length; i++) {
          if (locations[i].name >= event.data.name) {
            break;
          }
        }
        locations.splice(i, 0, event.data);
        return locations;
      } else if (event.modification === 'DELETE') {
        return locations.filter((location) => location.id !== event.data.id);
      }
    };

    // subscribe to location changes
    const subscription = handle.useSubscription(
      {
        query: `
      subscription {
        locationSubscription {
          modification,
          data {
            id,
            name,
            address
          }
        }
      }
      `,
      },
      handleSubscription
    );

    // hack to get things working
    return {
      base: result,
      live: subscription.data,
    };
  },
};
</script>